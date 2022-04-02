use std::fs;
use std::path::Path;
use std::io::prelude::*;
use std::io::Cursor;
use std::collections::HashMap;
use quick_xml::{Reader, Writer};
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText, BytesDecl};
use regex::Regex;
use super::etreenode::ETreeNode;

/// Element tree
///
/// `etree.ETree` stores a sequence of `etree.ETreeNode`.
#[derive(Debug, Clone)]
pub struct ETree {
    indent:String,
    count:usize,
    version:Vec<u8>,
    encoding:Option<Vec<u8>>,
    standalone:Option<Vec<u8>>,
    data:Vec<ETreeNode>,
    crlf:String,
}

impl ETree {
    #[allow(dead_code)]
    pub fn parse_file<P:AsRef<Path>>(path:P) -> ETree {
        let mut fh = fs::OpenOptions::new().read(true).open(path).expect(
            "Could not open file",
        );
        let mut buf = String::new();
        fh.read_to_string(&mut buf).expect("Could not read file");
        ETree::parse_str(buf.as_str())
    }
    #[allow(dead_code)]
    pub fn parse_str(content:&str) -> ETree {
        let fileformat = if content.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        let mut out = ETree {
            indent:"".to_string(),
            count:0,
            version: Vec::new(),
            encoding: None,
            standalone: None,
            data: Vec::new(),
            crlf: fileformat.to_string(),
        };
        out.read(content);
        out.detect_indent();
        out
    }
    #[allow(dead_code)]
    pub fn write_file<P:AsRef<Path>>(&self, path:P) -> std::io::Result<()> {
        fs::write(path, self.write())
    }
    #[allow(dead_code)]
    /// get XML version
    pub fn get_version(&self) -> Option<String> {
        String::from_utf8(self.version.clone()).ok()
    }
    #[allow(dead_code)]
    /// set XML version
    pub fn set_version(&mut self, version:&str) {
        self.version = version.to_string().into_bytes();
    }
    #[allow(dead_code)]
    /// get XML encoding
    pub fn get_encoding(&self) -> Option<String> {
        self.encoding.as_ref().and_then(|x| String::from_utf8(x.to_vec()).ok())
    }
    #[allow(dead_code)]
    /// set XML encoding
    pub fn set_encoding(&mut self, encoding:&str) {
        self.encoding = Some(encoding.to_string().into_bytes());
    }
    #[allow(dead_code)]
    /// get XML standalone
    pub fn get_standalone(&self) -> Option<String> {
        self.standalone.as_ref().and_then(|x| String::from_utf8(x.to_vec()).ok())
    }
    #[allow(dead_code)]
    /// set XML standalone
    pub fn set_standalone(&mut self, standalone:&str) {
        self.standalone = Some(standalone.to_string().into_bytes());
    }
    #[allow(dead_code)]
    /// get position of root node
    pub fn root(&self) -> usize {
        let mut idx = 0;
        while idx < self.data.len() {
            if !(self.data[idx].get_localname().starts_with("<") && self.data[idx].get_localname().ends_with(">")) {
                break;
            }
            idx += 1;
        }
        idx
    }
    #[allow(dead_code)]
    /// get position of parent node
    pub fn parent(&self, pos:usize) -> Option<usize> {
        if pos <= 0 || pos >= self.data.len() {
            None
        } else {
            let close_tag = Regex::new(r"^(?P<parent>#.*?)(?P<current>\d+)#$").unwrap();
            if let Some(c) = close_tag.captures(&self.data[pos].get_route()) {
                let route = c.name("parent").unwrap().as_str();
                let mut pos2 = pos;
                while pos2 > 0 {
                    pos2 -= 1;
                    if self.data[pos2].get_route() == route {
                        return Some(pos2);
                    }
                }
            }
            None
        }
    }
    #[allow(dead_code)]
    /// get positions of children node
    pub fn children(&self, pos:usize) -> Vec<usize> {
        let mut out:Vec<usize> = Vec::new();
        if pos < self.data.len() {
            let route = format!("{}{}#", self.data[pos].get_route(), self.data[pos].get_idx());
            for i in pos..self.data.len() {
                if self.data[i].get_route() == route {
                    out.push(i);
                }
            }
        }
        out
    }
    #[allow(dead_code)]
    /// get positions of descendant node
    pub fn descendant(&self, pos:usize) -> Vec<usize> {
        let mut out:Vec<usize> = Vec::new();
        if pos < self.data.len() {
            let route = format!("{}{}#", self.data[pos].get_route(), self.data[pos].get_idx());
            for i in pos..self.data.len() {
                if self.data[i].get_route().starts_with(&route) {
                    out.push(i);
                }
            }
        }
        out
    }
    #[allow(dead_code)]
    /// get position of previous sibling node
    pub fn previous(&self, pos:usize) -> Option<usize> {
        if pos <= 0  || pos >= self.data.len() {
            None
        } else {
            let mut pos2 = pos;
            while pos2 > 0 {
                pos2 -= 1;
                if self.data[pos2].get_route() == self.data[pos].get_route() {
                    return Some(pos2);
                }
            }
            None
        }
    }
    #[allow(dead_code)]
    /// get position of next sibling node
    pub fn next(&self, pos:usize) -> Option<usize> {
        if pos >= self.data.len() - 1 {
            None
        } else {
            let mut pos2 = pos + 1;
            while pos2 < self.data.len() {
                if self.data[pos2].get_route() == self.data[pos].get_route() {
                    return Some(pos2);
                }
                pos2 += 1;
            }
            None
        }
    }
    #[allow(dead_code)]
    /// get position by idx
    pub fn pos(&self, idx:usize) -> Option<usize> {
        for i in 0..self.data.len() {
            if self.data[i].get_idx() == idx {
                return Some(i);
            }
        }
        None
    }
    #[allow(dead_code)]
    /// get node by position
    pub fn node(&self, idx:usize) -> Option<&ETreeNode> {
        self.data.get(idx)
    }
    #[allow(dead_code)]
    /// get mut node by position
    pub fn node_mut(&mut self, idx:usize) -> Option<&mut ETreeNode> {
        self.data.get_mut(idx)
    }
    #[allow(dead_code)]
    /// clone a subtree rooted at the node of specified position
    pub fn subtree(&self, pos:usize) -> ETree {
        let mut tree = ETree {
            indent:self.indent.clone(),
            count:0,
            version: self.version.clone(),
            encoding: self.encoding.clone(),
            standalone: self.standalone.clone(),
            data: Vec::new(),
            crlf: self.crlf.clone(),
        };
        let offspring = self.descendant(pos);
        let mut node = self.data[pos].clone();
        let base_root_len = node.get_route().len() - 1;
        node.set_route(node.get_route().get(base_root_len..).unwrap());
        tree.data.push(node);
        for i in offspring {
            node = self.data[i].clone();
            node.set_route(node.get_route().get(base_root_len..).unwrap());
            tree.data.push(node);
        }
        tree
    }
    #[allow(dead_code)]
    /// append sibling node before the node of specified position and return the position of sibling node
    ///
    /// *Warning*: position which is larger than return value and obtained before this function all should be re-obtained 
    pub fn append_previous_node(&mut self, pos:usize, mut node:ETreeNode) -> Option<usize> {
        if let Some(cell) = self.prepare_append_previous(pos) {
            node.set_idx(self.count);
            node.set_tail(&cell.get_tail());
            node.set_route(&cell.get_route());
            self.count += 1;
            self.data.insert(cell.get_idx(), node);
            Some(cell.get_idx())
        } else {
            None
        }
    }
    #[allow(dead_code)]
    /// append sibling node after the node of specified position and return the position of sibling node
    ///
    /// *Warning*: position which is larger than return value and obtained before this function all should be re-obtained 
    pub fn append_next_node(&mut self, pos:usize, mut node:ETreeNode) -> Option<usize> {
        if let Some(cell) = self.prepare_append_next(pos) {
            node.set_idx(self.count);
            node.set_tail(&cell.get_tail());
            node.set_route(&cell.get_route());
            self.count += 1;
            self.data.insert(cell.get_idx(), node);
            Some(cell.get_idx())
        } else {
            None
        }
    }
    #[allow(dead_code)]
    /// append child node below the node of specified position and return the position of child node
    ///
    /// *Warning*: position which is larger than return value and obtained before this function all should be re-obtained 
    pub fn append_child_node(&mut self, pos:usize, mut node:ETreeNode) -> Option<usize> {
        if let Some(cell) = self.prepare_append_child(pos) {
            node.set_idx(self.count);
            node.set_tail(&cell.get_tail());
            node.set_route(&cell.get_route());
            self.count += 1;
            self.data.insert(cell.get_idx(), node);
            Some(cell.get_idx())
        } else {
            None
        }
    }
    #[allow(dead_code)]
    /// append sibling tree before the node of specified position and return the position of sibling tree
    ///
    /// *Warning*: position which is larger than return value and obtained before this function all should be re-obtained 
    pub fn append_previous_tree(&mut self, pos:usize, mut tree:ETree) -> Option<usize> {
        if let Some(cell) = self.prepare_append_previous(pos) {
            let (startidx, endidx) = tree.subtree_reindex(self.count);
            if startidx == self.count {
                self.count = endidx;
            } else {
                let (_, _) = tree.subtree_reindex(startidx);
                let (_, endidx) = tree.subtree_reindex(self.count);
                self.count = endidx;
            }
            tree.data[0].set_tail(&cell.get_tail());
            for i in 0..tree.data.len() {
                let route = format!("{}{}", cell.get_route(), tree.data[i].get_route().get(1..).unwrap());
                tree.data[i].set_route(&route);
                self.data.insert(cell.get_idx() + i, tree.data[i].clone());
            }
            Some(cell.get_idx())
        } else {
            None
        }
    }
    #[allow(dead_code)]
    /// append sibling tree after the node of specified position and return the position of sibling tree
    ///
    /// *Warning*: position which is larger than return value and obtained before this function all should be re-obtained 
    pub fn append_next_tree(&mut self, pos:usize, mut tree:ETree) -> Option<usize> {
        if let Some(cell) = self.prepare_append_next(pos) {
            let (startidx, endidx) = tree.subtree_reindex(self.count);
            if startidx == self.count {
                self.count = endidx;
            } else {
                let (_, _) = tree.subtree_reindex(startidx);
                let (_, endidx) = tree.subtree_reindex(self.count);
                self.count = endidx;
            }
            tree.data[0].set_tail(&cell.get_tail());
            for i in 0..tree.data.len() {
                let route = format!("{}{}", cell.get_route(), tree.data[i].get_route().get(1..).unwrap());
                tree.data[i].set_route(&route);
                self.data.insert(cell.get_idx() + i, tree.data[i].clone());
            }
            Some(cell.get_idx())
        } else {
            None
        }
    }
    #[allow(dead_code)]
    /// append child tree below the node of specified position and return the position of child tree
    ///
    /// *Warning*: position which is larger than return value and obtained before this function all should be re-obtained 
    pub fn append_child_tree(&mut self, pos:usize, mut tree:ETree) -> Option<usize> {
        if let Some(cell) = self.prepare_append_child(pos) {
            let (startidx, endidx) = tree.subtree_reindex(self.count);
            if startidx == self.count {
                self.count = endidx;
            } else {
                let (_, _) = tree.subtree_reindex(startidx);
                let (_, endidx) = tree.subtree_reindex(self.count);
                self.count = endidx;
            }
            tree.data[0].set_tail(&cell.get_tail());
            for i in 0..tree.data.len() {
                let route = format!("{}{}", cell.get_route(), tree.data[i].get_route().get(1..).unwrap());
                tree.data[i].set_route(&route);
                self.data.insert(cell.get_idx() + i, tree.data[i].clone());
            }
            Some(cell.get_idx())
        } else {
            None
        }
    }
    #[allow(dead_code)]
    /// remove a subtree rooted at the node of specified position
    ///
    /// *Warning*: position which is larger than specified value and obtained before this function all should be re-obtained 
    pub fn remove(&mut self, pos:usize) {
        if let Some(previous) = self.previous(pos) {
            let tail = self.data[pos].get_tail();
            self.data[previous].set_tail(&tail);
        } else if let Some(_next) = self.next(pos) {
        } else if let Some(parent) = self.parent(pos) {
            let mut text = String::from(self.data[parent].get_text().as_deref().unwrap());
            if text.ends_with(&self.indent) {
                let retain = text.len() - self.indent.len();
                text.truncate(retain);
                self.data[parent].set_text(&text);
            }
        }
        let offspring = self.descendant(pos);
        let mut i = offspring.len();
        while i > 0 {
            i -= 1;
            self.data.remove(offspring[i]);
        }
        self.data.remove(pos);
    }
    #[allow(dead_code)]
    /// clear indent and return old indent
    pub fn noindent(&mut self) -> String {
        let oldindent = format!("{}{}", self.crlf, self.indent);
        self.indent = "".to_string();
        self.crlf = "".to_string();
        for item in self.data.iter_mut() {
            item.set_tail(item.get_tail().trim());
            if let Some(text) = item.get_text() {
                item.set_text(text.trim());
            }
        }
        oldindent
    }
    #[allow(dead_code)]
    /// format nodes according to indent
    pub fn pretty(&mut self, indent:&str) {
        self.set_indent(indent);
        let nodecnt = self.data.len();
        let mut idx = 0;
        while idx < nodecnt {
            if self.data[idx].get_localname().starts_with("<") && self.data[idx].get_localname().ends_with(">") {
                self.data[idx].set_tail(&self.crlf);
            } else {
                break;
            }
            idx += 1;
        }
        self.pretty_tree(idx, 0);
    }

    fn read(&mut self, data:&str) {
        let mut reader = Reader::from_str(data);
        let mut buf = Vec::new();
        let mut ns_buf = Vec::new();
        let mut status = 0;
        let mut route = "#".to_string();
        let close_tag = Regex::new(r"^(?P<parent>#.*?)(?P<current>\d+)#$").unwrap();
        let mut closeidx = 0;
        loop {
            match reader.read_namespaced_event(&mut buf, &mut ns_buf) {
                Ok((ref ns, Event::Start(ref e))) => {
                    status = 1;
                    let fulltag = String::from_utf8(e.name().to_vec()).unwrap();
                    let shorttag = String::from_utf8(e.local_name().to_vec()).unwrap();
                    let prefixlen = fulltag.len() - shorttag.len();
                    let prefix = if prefixlen > 0 {
                        fulltag.get(..prefixlen-1).unwrap().to_string()
                    } else {
                        "".to_string()
                    };
                    let mut node = ETreeNode::new(&shorttag);
                    node.set_idx(self.count);
                    if ns.is_some() {
                        node.set_namespace(&String::from_utf8(ns.unwrap().to_vec()).unwrap());
                    }
                    node.set_namespace_abbrev(&prefix);
                    node.set_text("");
                    node.set_route(&route);
                    for item in e.attributes() {
                        if let Ok(attr) = item {
                            node.set_attr(&String::from_utf8(attr.key.to_vec()).unwrap(), &attr.unescape_and_decode_value(&reader).unwrap());
                        }
                    }
                    self.data.push(node);
                    route = format!("{}{}#", route, self.count);
                    self.count += 1;
                },
                Ok((_, Event::End(_))) => {
                    status = 2;
                    if let Some(c) = close_tag.captures(route.clone().as_str()) {
                        route = c.name("parent").unwrap().as_str().to_string();
                        let current = c.name("current").unwrap().as_str();
                        closeidx = current.parse().unwrap();
                    }
                },
                Ok((ref ns, Event::Empty(ref e))) => {
                    status = 2;
                    let fulltag = String::from_utf8(e.name().to_vec()).unwrap();
                    let shorttag = String::from_utf8(e.local_name().to_vec()).unwrap();
                    let prefixlen = fulltag.len() - shorttag.len();
                    let prefix = if prefixlen > 0 {
                        fulltag.get(..prefixlen-1).unwrap().to_string()
                    } else {
                        "".to_string()
                    };
                    let mut node = ETreeNode::new(&shorttag);
                    node.set_idx(self.count);
                    if ns.is_some() {
                        node.set_namespace(&String::from_utf8(ns.unwrap().to_vec()).unwrap());
                    }
                    node.set_namespace_abbrev(&prefix);
                    node.set_route(&route);
                    for item in e.attributes() {
                        if let Ok(attr) = item {
                            node.set_attr(&String::from_utf8(attr.key.to_vec()).unwrap(), &attr.unescape_and_decode_value(&reader).unwrap());
                        }
                    }
                    self.data.push(node);
                    closeidx = self.count;
                    self.count += 1;
                },
                Ok((_, Event::Text(e))) => {
                    if status == 1 {
                        if let Some(node) = self.data.get_mut(self.count - 1) {
                            node.set_text(&e.unescape_and_decode(&reader).unwrap());
                        }
                    } else if status == 2 {
                        if let Some(node) = self.data.get_mut(closeidx) {
                            node.set_tail(&e.unescape_and_decode(&reader).unwrap());
                        }
                    }
                },
                Ok((_, Event::Comment(e))) => {
                    status = 2;
                    let mut node = ETreeNode::new("<Comment>");
                    node.set_idx(self.count);
                    node.set_text(&e.unescape_and_decode(&reader).unwrap());
                    node.set_route(&route);
                    self.data.push(node);
                    closeidx = self.count;
                    self.count += 1;
                },
                Ok((_, Event::CData(e))) => {
                    status = 2;
                    let mut node = ETreeNode::new("<CData>");
                    node.set_idx(self.count);
                    node.set_text(&e.unescape_and_decode(&reader).unwrap());
                    node.set_route(&route);
                    self.data.push(node);
                    closeidx = self.count;
                    self.count += 1;
                },
                Ok((_, Event::Decl(ref e))) => {
                    self.version = e.version().unwrap().into_owned();
                    if let Some(x) = e.encoding() {
                        self.encoding = Some(x.unwrap().into_owned());
                    }
                    if let Some(x) = e.standalone() {
                        self.standalone = Some(x.unwrap().into_owned());
                    }
                },
                Ok((_, Event::PI(e))) => {
                    status = 2;
                    let mut node = ETreeNode::new("<PI>");
                    node.set_idx(self.count);
                    node.set_text(&e.unescape_and_decode(&reader).unwrap());
                    node.set_route(&route);
                    self.data.push(node);
                    closeidx = self.count;
                    self.count += 1;
                },
                Ok((_, Event::DocType(e))) => {
                    status = 2;
                    let mut node = ETreeNode::new("<DocType>");
                    node.set_idx(self.count);
                    node.set_text(&e.unescape_and_decode(&reader).unwrap());
                    node.set_route(&route);
                    self.data.push(node);
                    closeidx = self.count;
                    self.count += 1;
                },
                Ok((_, Event::Eof)) => break,
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            }
        }
    }
    fn write(&self) -> Vec<u8> {
        let close_tag = Regex::new(r"^(?P<parent>#.*?)(?P<current>\d+)#$").unwrap();
        let mut idxmap:HashMap<String, usize> = HashMap::new();
        for idx in 0..self.data.len() {
            idxmap.insert(self.data[idx].get_idx().to_string(), idx);
        }
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        let elem = BytesDecl::new(self.version.as_slice(),
                                  self.encoding.as_deref(),
                                  self.standalone.as_deref());
        let _ = writer.write_event(Event::Decl(elem));
        let _ = writer.write(self.crlf.as_bytes());
        let nodelen = self.data.len();
        for idx in 0..nodelen {
            if idx > 0 {
                if self.data[idx].get_route() == self.data[idx-1].get_route() {
                    // Sibling node for last node
                    if self.data[idx-1].get_text().is_some() {
                        if !(self.data[idx-1].get_localname().starts_with("<") && self.data[idx-1].get_localname().ends_with(">")) {
                            let elem = BytesEnd::owned(Vec::<u8>::from(self.data[idx-1].get_name()));
                            assert!(writer.write_event(Event::End(elem)).is_ok());
                        }
                        let elem = BytesText::from_plain_str(self.data[idx-1].get_tail().as_str()).into_owned();
                        assert!(writer.write_event(Event::Text(elem)).is_ok());
                    }
                } else if self.data[idx].get_route().starts_with(&self.data[idx-1].get_route()) {
                    // Child node for last node
                } else if self.data[idx-1].get_route().starts_with(&self.data[idx].get_route()) {
                    // Close tag
                    if self.data[idx-1].get_text().is_some() {
                        if !(self.data[idx-1].get_localname().starts_with("<") && self.data[idx-1].get_localname().ends_with(">")) {
                            let elem = BytesEnd::owned(Vec::<u8>::from(self.data[idx-1].get_name()));
                            assert!(writer.write_event(Event::End(elem)).is_ok());
                        }
                        let elem = BytesText::from_plain_str(self.data[idx-1].get_tail().as_str()).into_owned();
                        assert!(writer.write_event(Event::Text(elem)).is_ok());
                    }
                    let mut route = self.data[idx-1].get_route();
                    while let Some(c) = close_tag.captures(&route.clone()) {
                        route = c.name("parent").unwrap().as_str().to_string();
                        let current = c.name("current").unwrap().as_str().to_string();
                        let closeidx = idxmap.get(&current).unwrap();
                        if !(self.data[*closeidx].get_localname().starts_with("<") && self.data[*closeidx].get_localname().ends_with(">")) {
                            let elem = BytesEnd::owned(Vec::<u8>::from(self.data[*closeidx].get_name()));
                            assert!(writer.write_event(Event::End(elem)).is_ok());
                        }
                        let elem = BytesText::from_plain_str(self.data[*closeidx].get_tail().as_str()).into_owned();
                        assert!(writer.write_event(Event::Text(elem)).is_ok());
                        if route == self.data[idx].get_route() {
                            break;
                        }
                    }
                } else {
                    panic!("Error route: {}[{}] {}[{}]", idx-1, self.data[idx-1].get_route(), idx, self.data[idx].get_route());
                }
            }
            if self.data[idx].get_localname() == "<Comment>" {
                let elem = BytesText::from_plain_str(self.data[idx].get_text().as_deref().unwrap()).into_owned();
                assert!(writer.write_event(Event::Comment(elem)).is_ok());
            } else if self.data[idx].get_localname() == "<CData>" {
                let elem = BytesText::from_plain_str(self.data[idx].get_text().as_deref().unwrap()).into_owned();
                assert!(writer.write_event(Event::CData(elem)).is_ok());
            } else if self.data[idx].get_localname() == "<PI>" {
                let elem = BytesText::from_plain_str(self.data[idx].get_text().as_deref().unwrap()).into_owned();
                assert!(writer.write_event(Event::PI(elem)).is_ok());
            } else if self.data[idx].get_localname() == "<DocType>" {
                let elem = BytesText::from_plain_str(self.data[idx].get_text().as_deref().unwrap()).into_owned();
                assert!(writer.write_event(Event::DocType(elem)).is_ok());
            } else {
                let name = self.data[idx].get_name();
                let mut elem = BytesStart::borrowed(name.as_bytes(), name.len());
                for attr in self.data[idx].get_attr_iter() {
                    elem.push_attribute((attr.0.as_str(), attr.1.as_str()));
                }
                if self.data[idx].get_text().is_some() {
                    assert!(writer.write_event(Event::Start(elem)).is_ok());
                    let elem = BytesText::from_plain_str(self.data[idx].get_text().as_deref().unwrap()).into_owned();
                    assert!(writer.write_event(Event::Text(elem)).is_ok());
                } else {
                    assert!(writer.write_event(Event::Empty(elem)).is_ok());
                    let elem = BytesText::from_plain_str(self.data[idx].get_tail().as_str()).into_owned();
                    assert!(writer.write_event(Event::Text(elem)).is_ok());
                }
            }
        }
        // Close all remaining tags
        if self.data[nodelen-1].get_text().is_some() {
            if !(self.data[nodelen-1].get_localname().starts_with("<") && self.data[nodelen-1].get_localname().ends_with(">")) {
                let elem = BytesEnd::owned(Vec::<u8>::from(self.data[nodelen-1].get_name()));
                assert!(writer.write_event(Event::End(elem)).is_ok());
            }
            let elem = BytesText::from_plain_str(self.data[nodelen-1].get_tail().as_str()).into_owned();
            assert!(writer.write_event(Event::Text(elem)).is_ok());
        }
        let mut route = self.data[nodelen-1].get_route();
        while let Some(c) = close_tag.captures(&route.clone()) {
            route = c.name("parent").unwrap().as_str().to_string();
            let current = c.name("current").unwrap().as_str().to_string();
            let closeidx = idxmap.get(&current).unwrap();
            if !(self.data[*closeidx].get_localname().starts_with("<") && self.data[*closeidx].get_localname().ends_with(">")) {
                let elem = BytesEnd::owned(Vec::<u8>::from(self.data[*closeidx].get_name()));
                assert!(writer.write_event(Event::End(elem)).is_ok());
            }
            let elem = BytesText::from_plain_str(self.data[*closeidx].get_tail().as_str()).into_owned();
            assert!(writer.write_event(Event::Text(elem)).is_ok());
            if route == "#" {
                break;
            }
        }
        writer.into_inner().into_inner()
    }
    fn detect_indent(&mut self) {
        let mut idx = self.data.len();
        while idx > 0 {
            idx -= 1;
            if !(self.data[idx].get_localname().starts_with("<") && self.data[idx].get_localname().ends_with(">")) {
                break;
            }
        }
        if let Some(previous) = self.previous(idx) {
            if self.data[previous].get_tail().starts_with(&self.data[idx].get_tail()) {
                self.indent = self.data[previous].get_tail().get(self.data[idx].get_tail().len()..).unwrap().to_string();
            }
        } else if let Some(parent) = self.parent(idx) {
            let text = String::from(self.data[parent].get_text().as_deref().unwrap());
            if text.starts_with(&self.data[idx].get_tail()) {
                self.indent = text.get(self.data[idx].get_tail().len()..).unwrap().to_string();
            }
        }
    }
    fn prepare_append_previous(&mut self, pos:usize) -> Option<ETreeNode> {
        if pos >= self.data.len() {
            None
        } else {
            if let Some(prev) = self.previous(pos) {
                self.prepare_append_next(prev)
            } else if let Some(parent) = self.parent(pos) {
                let mut node = ETreeNode::new("");
                node.set_tail(&String::from(self.data[parent].get_text().as_deref().unwrap()));
                node.set_route(&format!("{}{}#", self.data[parent].get_route(), self.data[parent].get_idx()));
                let newpos = parent + 1;
                node.set_idx(newpos);
                Some(node)
            } else {
                None
            }
        }
    }
    fn prepare_append_next(&mut self, pos:usize) -> Option<ETreeNode> {
        if pos >= self.data.len() {
            None
        } else {
            let mut node = ETreeNode::new("");
            node.set_tail(&self.data[pos].get_tail());
            node.set_route(&self.data[pos].get_route());
            if let Some(prev) = self.previous(pos) {
                let tail = self.data[prev].get_tail();
                self.data[pos].set_tail(&tail);
            } else if let Some(parent) = self.parent(pos) {
                let tail = String::from(self.data[parent].get_text().as_deref().unwrap());
                self.data[pos].set_tail(&tail);
            }
            let offspring = self.descendant(pos);
            let newpos = if offspring.len() == 0 {
                pos + 1
            } else {
                offspring[offspring.len()-1] + 1
            };
            node.set_idx(newpos);
            Some(node)
        }
    }
    fn prepare_append_child(&mut self, pos:usize) -> Option<ETreeNode> {
        if pos >= self.data.len() {
            return None;
        }
        let mut node = ETreeNode::new("");
        node.set_route(&format!("{}{}#", self.data[pos].get_route(), self.data[pos].get_idx()));
        let children = self.children(pos);
        match children.len() {
            0 => {
                // No child exists
                let previous = self.previous(pos);
                let tail = if previous.is_some() {
                    format!("{}", self.data[previous.unwrap()].get_tail())
                } else {
                    let parent = self.parent(pos);
                    if parent.is_some() {
                        format!("{}", self.data[parent.unwrap()].get_tail())
                    } else {
                        self.crlf.clone()
                    }
                };
                let text = format!("{}{}", tail, self.indent);
                node.set_tail(&tail);
                if self.data[pos].get_text().is_none() {
                    self.data[pos].set_text(&text);
                } else if self.data[pos].get_text().as_deref() == Some("") {
                    self.data[pos].set_text(&text);
                }
                node.set_idx(pos + 1);
            },
            _ => {
                let previous = children[children.len()-1];
                node.set_tail(&self.data[previous].get_tail());
                if let Some(previous2) = self.previous(previous) {
                    let tail = self.data[previous2].get_tail();
                    self.data[previous].set_tail(&tail);
                } else {
                    let parent = self.parent(previous).unwrap();
                    let tail = self.data[parent].get_tail();
                    self.data[previous].set_tail(&tail);
                }
                let offspring = self.descendant(pos);
                node.set_idx(offspring[offspring.len()-1]+1);
           },
        }
        Some(node)
    }
    fn subtree_reindex(&mut self, start_idx:usize) -> (usize, usize) {
        let datacnt = self.data.len();
        if datacnt > 0 {
            let mut idx_min = self.data[0].get_idx();
            let mut idx_max = self.data[0].get_idx();
            let mut idx_cnt = 1;
            for i in 1..datacnt {
                if self.data[i].get_idx() > idx_max {
                    idx_max = self.data[i].get_idx();
                }
                if self.data[i].get_idx() < idx_min {
                    idx_min = self.data[i].get_idx();
                }
                idx_cnt += 1;
            }
            if (start_idx + idx_cnt <= idx_min) || (start_idx > idx_max) {
                let mut idx_cur = start_idx;
                for i in 0..datacnt {
                    let idx_old = self.data[i].get_idx();
                    self.data[i].set_idx(idx_cur);
                    for j in 0..datacnt {
                        let route = self.data[j].get_route().replace(format!("#{}#", idx_old).as_str(), format!("#{}#", idx_cur).as_str());
                        self.data[j].set_route(&route);
                    }
                    idx_cur += 1;
                }
                (start_idx, idx_cur)
            } else {
                (idx_max + datacnt + 1, idx_max + datacnt * 2 + 1)
            }
        } else {
            (0, 0)
        }
    }
    fn set_indent(&mut self, indent:&str) {
        let lines:Vec<&str> = indent.lines().collect();
        if lines.len() >= 2 && lines[lines.len() - 1].len() > 0 {
            if indent.contains("\r\n") {
                self.crlf = "\r\n".to_string();
            } else if indent.contains("\n") {
                self.crlf = "\n".to_string();
            } else {
                self.crlf = "\r".to_string();
            }
        } else {
            self.crlf = "\n".to_string();
        }
        self.indent = lines[lines.len() - 1].to_string();
    }
    fn pretty_tree(&mut self, pos:usize, level:usize) {
        let tail = format!("{}{}", self.crlf, self.indent.repeat(level));
        self.data[pos].set_tail(&tail);
        let children = self.children(pos);
        if children.len() > 0 {
            let text = format!("{}{}{}",
                self.data[pos].get_text().as_deref().unwrap().trim(),
                self.crlf.as_str(),
                self.indent.repeat(level+1));
            self.data[pos].set_text(&text);
            for subpos in children.iter() {
                self.pretty_tree(*subpos, level+1);
            }
            self.data[children[children.len()-1]].set_tail(&tail);
        } else {
            if !(self.data[pos].get_localname().starts_with("<") && self.data[pos].get_localname().ends_with(">")) {
                let text = format!("{}", self.data[pos].get_text().as_deref().unwrap().trim());
                self.data[pos].set_text(&text);
            }
        }
    }
}

/// transform root node into a tree
impl From<ETreeNode> for ETree {
    fn from(mut node:ETreeNode) -> Self {
        let mut tree = ETree {
            indent:"".to_string(),
            count:1,
            version:"1.0".to_string().into_bytes(),
            encoding:None,
            standalone:None,
            data:Vec::new(),
            crlf:"".to_string(),
        };
        node.set_idx(0);
        node.set_route("#");
        tree.data.push(node);
        tree
    }
}
