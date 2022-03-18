use regex::Regex;
use super::etree::ETree;

/// XPath operation
///
/// # Supported syntax:
/// ## Node query
/// - `nodename`: the same as `//nodename`
/// - `*`: any node
/// - `/`: node in the children of current node
/// - `//`: node in the descendant of current node
/// - `.`: current node
/// - `..`: parent node
/// - `@attrname`
/// ## Node Predicate
/// - `[1]`: first element
/// - `[last()-1]`: second to last element
/// - `[position() < 3]`: first and second element
/// - `[@attrname]`: element with attr `attrname`
/// - `[@*]`: element with any attr
/// - `[@attrname='value']`: element with attr `attrname`=`value`
/// - `[text()='value']`: element which text is equal to `value`
/// - `[child-tag='value']`: element which contains child `child-tag` and child tag's text is equal to `value`
/// - `[text()='value' and child-tag='value']`: multiple condition with `and`/`or` and parenthesis
/// # Search algorithm
/// 1. `path` is split into multiple parts by consecutive "/".
///    - e.g. "//tag1/tag2[text()='abc']" is split into ["//tag1", "/tag2[text()='abc']"]
/// 2. find first part from the specified node
/// 3. find next part from the result of last find
/// 4. repeat step 3 until all part finished
pub trait XPath {
    /// find `path` from the root node
    fn find(&self, path:&str) -> Vec<usize> {
        self.find_at(path, 0)
    }
    /// find `path` from specified node
    fn find_at(&self, path:&str, pos:usize) -> Vec<usize> {
        let re = Regex::new(r"/+").unwrap();
        let newpath = re.replace_all(path, "<>$0");
        let mut path_todo:Vec<String> = newpath.split("<>").map(|x| x.to_string()).collect();
        if path_todo[0] == "" || path_todo[0] == "." {
            path_todo.remove(0);
        } else if path_todo[0] == "/" {
        } else if path_todo[0] == ".." {
            path_todo.remove(0);
            path_todo.insert(0, "/..".to_string());
        } else {
            let element = path_todo.remove(0);
            path_todo.insert(0, format!("//{}", element));
        }
        let mut pos_todo:Vec<usize> = vec![pos];
        for pathitem in path_todo {
            let pos_doing = pos_todo;
            pos_todo = Vec::new();
            for positem in pos_doing {
                let mut result = self.find_at_action(&pathitem, positem);
                pos_todo.append(&mut result);
            }
        }
        pos_todo
    }
    /// find one part of path from specified node
    fn find_at_action(&self, path:&str, pos:usize) -> Vec<usize>;
}

impl XPath for ETree {
    fn find(&self, path:&str) -> Vec<usize> {
        self.find_at(path, self.root())
    }

    fn find_at_action(&self, path:&str, pos:usize) -> Vec<usize> {
        let mut result:Vec<usize> = Vec::new();
        if path == "/." {
            result.push(pos);
        } else if path == "/.." {
            if let Some(parent) = self.parent(pos) {
                result.push(parent);
            }
        } else {
            let re = Regex::new(r"^(/+)(.+)$").unwrap();
            if let Some(c) = re.captures(path) {
                let m1 = c.get(1).unwrap().as_str();
                let m2 = c.get(2).unwrap().as_str();
                let container = if m1 == "//" {
                    self.descendant(pos)
                } else { /* "/" */
                    self.children(pos)
                };
                if m2.starts_with("@") {
                    let attr = m2.get(1..).unwrap();
                    if attr == "*" {
                        for positem in container {
                            if self.node(positem).unwrap().get_attr_count() > 0 {
                                result.push(positem);
                            }
                        }
                    } else {
                        for positem in container {
                            if self.node(positem).unwrap().get_attr(attr).is_some() {
                                result.push(positem);
                            }
                        }
                    }
                } else {
                    let re = Regex::new(r"^(.+?)(?:\[(.+?)\])?$").unwrap();
                    if let Some(c) = re.captures(m2) {
                        let tag = c.get(1).unwrap().as_str();
                        let mut container:Vec<usize> = container.iter().filter(|&x| self.node(*x).unwrap().get_name()==tag).map(|x| *x).collect();
                        if let Some(predicate) = c.get(2) {
                            let pat1 = Regex::new(r"\band\b").unwrap();
                            let pat2 = Regex::new(r"\bor\b").unwrap();
                            let expr = pat2.replace_all(pat1.replace_all(predicate.as_str(), "&&").into_owned().as_str(), "||").into_owned();
                            let expr = expr.replace("=", "==").replace("!==", "!=").replace(">==", ">=").replace("<==", "<=");
                            let re = Regex::new(r"((?P<attr>@\S+?)|(?P<func>\S+?\s*\(\s*\))|(?P<tag>\S+?))\s*=").unwrap();
                            let mut params_attr:Vec<String> = Vec::new();
                            let mut params_func:Vec<String> = Vec::new();
                            let mut params_tag:Vec<String> = Vec::new();
                            for param in re.captures_iter(&expr) {
                                if param.name("attr").is_some() {
                                    let x = param.name("attr").unwrap().as_str().to_string();
                                    if !params_attr.contains(&x) {
                                        params_attr.push(x);
                                    }
                                } else if param.name("func").is_some() {
                                    let x = param.name("func").unwrap().as_str().to_string();
                                    if !params_func.contains(&x) {
                                        params_func.push(x);
                                    }
                                } else if param.name("tag").is_some() {
                                    let x = param.name("tag").unwrap().as_str().to_string();
                                    if !params_tag.contains(&x) {
                                        params_tag.push(x);
                                    }
                                }
                            }
                            let container_len = container.len();
                            for i in 0..container_len {
                                let mut found = true;
                                let mut cur_expr = expr.clone();
                                for param in params_attr.iter() {
                                    if let Some(v) = self.node(container[i]).unwrap().get_attr(param.get(1..).unwrap()) {
                                        cur_expr = cur_expr.replace(param.as_str(), format!("'{}'", v).as_str());
                                    } else {
                                        found = false;
                                        break;
                                    }
                                }
                                if !found {
                                    break;
                                }
                                for param in params_func.iter() {
                                    if param.starts_with("text") {
                                        cur_expr = cur_expr.replace(param.as_str(), format!("'{}'", self.node(container[i]).unwrap().get_text().unwrap_or("".to_string())).as_str());
                                    } else if param.starts_with("position") {
                                        cur_expr = cur_expr.replace(param.as_str(), format!("{}", i+1).as_str());
                                    } else if param.starts_with("last") {
                                        cur_expr = cur_expr.replace(param.as_str(), format!("{}", container_len).as_str());
                                    }
                                }
                                if params_tag.len() > 0 {
                                    let mut subfound:Vec<Vec<usize>> = Vec::new();
                                    let mut curcomb:Vec<usize> = Vec::new();
                                    for _ in 0..params_tag.len() {
                                        subfound.push(Vec::new());
                                        curcomb.push(0);
                                    }
                                    let subchildren = self.children(container[i]);
                                    for subi in subchildren {
                                        for subj in 0..params_tag.len() {
                                            if self.node(subi).unwrap().get_name() == params_tag[subj] {
                                                subfound[subj].push(subi);
                                            }
                                        }
                                    }
                                    if subfound.iter().all(|ref x| x.len() > 0) {
                                        let backup_expr = cur_expr;
                                        let mut exit_flag = false;
                                        loop {
                                            cur_expr = backup_expr.clone();
                                            for subj in 0..params_tag.len() {
                                                cur_expr = cur_expr.replace(params_tag[subj].as_str(),
                                                    format!("'{}'",
                                                        self.node(subfound[subj][curcomb[subj]]).unwrap().get_text().unwrap_or("".to_string())).as_str());
                                            }
                                            if eval::eval(cur_expr.as_str()) == Ok(eval::to_value(true)) {
                                                result.push(container[i]);
                                                break;
                                            }
                                            let mut subi = curcomb.len() - 1;
                                            loop {
                                                curcomb[subi] += 1;
                                                if curcomb[subi] >= subfound[subi].len() {
                                                    curcomb[subi] = 0;
                                                    if subi > 0 {
                                                        subi -= 1;
                                                    } else {
                                                        exit_flag = true;
                                                        break;
                                                    }
                                                } else {
                                                    break;
                                                }
                                            }
                                            if exit_flag {
                                                break;
                                            }
                                        }
                                    }
                                } else {
                                    if eval::eval(cur_expr.as_str()) == Ok(eval::to_value(true)) {
                                        result.push(container[i]);
                                    }
                                }
                            }
                        } else {
                            result.append(&mut container);
                        }
                    } else {
                        // Syntax error
                    }
                }
            } else {
                // Syntax error
            }
        }
        result
    }
}
