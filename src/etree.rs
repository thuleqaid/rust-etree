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
    enable_index:bool,
    index:HashMap<usize, usize>,
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
            enable_index: false,
            index: HashMap::new(),
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
    /// get whether index feature is enabled
    pub fn get_enable_index(&self) -> bool {
        self.enable_index
    }
    #[allow(dead_code)]
    /// set whether index feature is enabled (usable for function `pos()`)
    pub fn set_enable_index(&mut self, enable_index:bool) {
        self.enable_index = enable_index;
        self.generate_index();
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
            for i in pos+1..self.data.len() {
                let curroute = self.data[i].get_route();
                if curroute == route {
                    out.push(i);
                } else if !curroute.starts_with(&route) {
                    break;
                }
            }
        }
        out
    }
    #[allow(dead_code)]
    /// get positions of children node with specified name
    pub fn children_by_name(&self, pos:usize, tagname:&str) -> Vec<usize> {
        let mut out:Vec<usize> = Vec::new();
        for i in self.children(pos) {
            if self.data[i].get_name() == tagname {
                out.push(i);
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
            for i in pos+1..self.data.len() {
                if self.data[i].get_route().starts_with(&route) {
                    out.push(i);
                } else {
                    break;
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
            let route = self.data[pos].get_route();
            while pos2 > 0 {
                pos2 -= 1;
                let curroute = self.data[pos2].get_route();
                if curroute == route {
                    return Some(pos2);
                }
                if !curroute.starts_with(&route) {
                    break;
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
            let route = self.data[pos].get_route();
            while pos2 < self.data.len() {
                let curroute = self.data[pos2].get_route();
                if curroute == route {
                    return Some(pos2);
                }
                if !curroute.starts_with(&route) {
                    break;
                }
                pos2 += 1;
            }
            None
        }
    }
    #[allow(dead_code)]
    /// get position by idx
    pub fn pos(&self, idx:usize) -> Option<usize> {
        if self.enable_index {
            self.index.get(&idx).copied()
        } else {
            for i in 0..self.data.len() {
                if self.data[i].get_idx() == idx {
                    return Some(i);
                }
            }
            None
        }
    }
    #[allow(dead_code)]
    /// get node by position
    pub fn node(&self, pos:usize) -> Option<&ETreeNode> {
        self.data.get(pos)
    }
    #[allow(dead_code)]
    /// get mut node by position
    pub fn node_mut(&mut self, pos:usize) -> Option<&mut ETreeNode> {
        self.data.get_mut(pos)
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
            enable_index: false,
            index: HashMap::new(),
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
            self.data.insert(cell.get_idx(), node);
            self.index.insert(self.count, cell.get_idx());
            self.update_index(cell.get_idx() + 1);
            self.count += 1;
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
            self.data.insert(cell.get_idx(), node);
            self.index.insert(self.count, cell.get_idx());
            self.update_index(cell.get_idx() + 1);
            self.count += 1;
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
            self.data.insert(cell.get_idx(), node);
            self.index.insert(self.count, cell.get_idx());
            self.update_index(cell.get_idx() + 1);
            self.count += 1;
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
                self.index.insert(tree.data[i].get_idx(), cell.get_idx() + i);
            }
            self.update_index(cell.get_idx() + tree.data.len());
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
                self.index.insert(tree.data[i].get_idx(), cell.get_idx() + i);
            }
            self.update_index(cell.get_idx() + tree.data.len());
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
                self.index.insert(tree.data[i].get_idx(), cell.get_idx() + i);
            }
            self.update_index(cell.get_idx() + tree.data.len());
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
            self.index.remove(&self.data[offspring[i]].get_idx());
            self.data.remove(offspring[i]);
        }
        self.index.remove(&self.data[pos].get_idx());
        self.data.remove(pos);
        self.update_index(pos);
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
    fn generate_index(&mut self) {
        if self.enable_index {
            self.index = HashMap::new();
            for i in 0..self.data.len() {
                self.index.insert(self.data[i].get_idx(), i);
            }
        }
    }
    fn update_index(&mut self, pos:usize) {
        if self.enable_index {
            for i in pos..self.data.len() {
                if let Some(x) = self.index.get_mut(&self.data[i].get_idx()) {
                    *x = i;
                }
            }
        }
    }
    #[allow(dead_code)]
    /// find the first node that matches `path` from the root node
    pub fn find(&self, path:&str) -> Option<usize> {
        self.find_at(path, self.root())
    }
    #[allow(dead_code)]
    /// find the first node that matches `path` from specified node
    pub fn find_at(&self, path:&str, pos:usize) -> Option<usize> {
        let mut iter = self.find_at_iter(path, pos);
        iter.next()
    }
    #[allow(dead_code)]
    /// find nodes that matches `path` from the root node
    pub fn find_iter(&self, path:&str) -> XPathIterator {
        self.find_at_iter(path, self.root())
    }
    #[allow(dead_code)]
    /// find nodes that matches `path` from specified node
    pub fn find_at_iter(&self, path:&str, pos:usize) -> XPathIterator {
        XPathIterator::new(self, path, pos, true)
    }
    #[allow(dead_code)]
    /// find the last node that matches `path` from the root node
    pub fn rfind(&self, path:&str) -> Option<usize> {
        self.rfind_at(path, self.root())
    }
    #[allow(dead_code)]
    /// find the last node that matches `path` from specified node
    pub fn rfind_at(&self, path:&str, pos:usize) -> Option<usize> {
        let mut iter = self.rfind_at_iter(path, pos);
        iter.next()
    }
    #[allow(dead_code)]
    /// find nodes in reverse order that matches `path` from the root node
    pub fn rfind_iter(&self, path:&str) -> XPathIterator {
        self.rfind_at_iter(path, self.root())
    }
    #[allow(dead_code)]
    /// find nodes in reverse order that matches `path` from specified node
    pub fn rfind_at_iter(&self, path:&str, pos:usize) -> XPathIterator {
        XPathIterator::new(self, path, pos, false)
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
            enable_index: false,
            index: HashMap::new(),
        };
        node.set_idx(0);
        node.set_route("#");
        tree.data.push(node);
        tree
    }
}

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
pub struct XPathIterator<'a> {
    tree: &'a ETree,
    direction: bool,
    path_list: Vec<String>,
    todo_list: Vec<(usize, usize)>,
}

impl<'a> XPathIterator<'a> {
    #[allow(dead_code)]
    fn new(tree:&'a ETree, path:&str, pos:usize, dir:bool) -> Self {
        let quote = vec!['\'', '"'];
        let enclose_open = vec!['['];
        let enclose_close = vec![']'];
        let mut escaped = false;
        let mut split_pos = Vec::new();
        let mut stack1 = Vec::new();
        let mut stack2 = Vec::new();
        for item in path.char_indices() {
            if escaped {
                escaped = false;
            } else {
                if item.1 == '\\' {
                    escaped = true;
                } else {
                    if quote.contains(&item.1) {
                        if stack1.is_empty() {
                            stack1.push(item.1);
                        } else {
                            if stack1[stack1.len()-1] == item.1 {
                                stack1.pop();
                            } else {
                                stack1.push(item.1);
                            }
                        }
                    } else if stack1.is_empty() {
                        if enclose_open.contains(&item.1) {
                            stack2.push(item.1);
                        } else if enclose_close.contains(&item.1) {
                            let mut p=0;
                            while p<enclose_close.len() {
                                if enclose_close[p] == item.1 {
                                    break;
                                }
                                p+=1;
                            }
                            if stack2[stack2.len()-1] == enclose_open[p] {
                                stack2.pop();
                            } else {
                                stack2.push(item.1);
                            }
                        } else if item.1 == '/' && stack2.is_empty() {
                            if split_pos.is_empty() {
                                split_pos.push(item.0);
                            } else if split_pos[split_pos.len()-1] + 1 < item.0 {
                                split_pos.push(item.0);
                            }
                        }
                    }
                }
            }
        }
        assert!(stack1.is_empty());
        assert!(stack2.is_empty());
        let mut path_todo:Vec<String> = Vec::new();
        if split_pos.is_empty() {
            path_todo.push(path.to_string());
        } else {
            let mut pos1 = 0;
            let mut posidx = 0;
            while posidx < split_pos.len() {
                path_todo.push(path.get(pos1..split_pos[posidx]).unwrap().to_string());
                pos1 = split_pos[posidx];
                posidx += 1;
            }
            path_todo.push(path.get(pos1..).unwrap().to_string());
        }
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
        Self {
            tree: tree,
            direction: dir,
            path_list: path_todo,
            todo_list: vec![(pos, 0)],
        }
    }
    fn _find(&self, path:&str, pos:usize) -> Vec<usize> {
        let mut result:Vec<usize> = Vec::new();
        if path == "/." {
            result.push(pos);
        } else if path == "/.." {
            if let Some(parent) = self.tree.parent(pos) {
                result.push(parent);
            }
        } else {
            let re = Regex::new(r"^(/+)(.+)$").unwrap();
            if let Some(c) = re.captures(path) {
                let m1 = c.get(1).unwrap().as_str();
                let m2 = c.get(2).unwrap().as_str();
                let container = if m1 == "//" {
                    self.tree.descendant(pos)
                } else { /* "/" */
                    self.tree.children(pos)
                };
                if m2.starts_with("@") {
                    let attr = m2.get(1..).unwrap();
                    if attr == "*" {
                        for positem in container {
                            if self.tree.node(positem).unwrap().get_attr_count() > 0 {
                                result.push(positem);
                            }
                        }
                    } else {
                        for positem in container {
                            if self.tree.node(positem).unwrap().get_attr(attr).is_some() {
                                result.push(positem);
                            }
                        }
                    }
                } else {
                    let re = Regex::new(r"^(.+?)(?:\[(.+?)\])?$").unwrap();
                    if let Some(c) = re.captures(m2) {
                        let tag = c.get(1).unwrap().as_str();
                        let mut container:Vec<usize> = container.iter().filter(|&x| self.tree.node(*x).unwrap().get_name()==tag).map(|x| *x).collect();
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
                                    if let Some(v) = self.tree.node(container[i]).unwrap().get_attr(param.get(1..).unwrap()) {
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
                                        cur_expr = cur_expr.replace(param.as_str(), format!("'{}'", self.tree.node(container[i]).unwrap().get_text().unwrap_or("".to_string())).as_str());
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
                                    let subchildren = self.tree.children(container[i]);
                                    for subi in subchildren {
                                        for subj in 0..params_tag.len() {
                                            if self.tree.node(subi).unwrap().get_name() == params_tag[subj] {
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
                                                        self.tree.node(subfound[subj][curcomb[subj]]).unwrap().get_text().unwrap_or("".to_string())).as_str());
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

impl<'a> Iterator for XPathIterator<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        while !self.todo_list.is_empty() {
            let item = self.todo_list.pop().unwrap();
            if item.1 >= self.path_list.len() {
                return Some(item.0);
            } else {
                let result = self._find(&self.path_list[item.1], item.0);
                let rlen = result.len();
                let mut ridx = rlen;
                if self.direction {
                    while ridx > 0 {
                        ridx -= 1;
                        self.todo_list.push((result[ridx], item.1+1));
                    }
                } else {
                    while ridx > 0 {
                        ridx -= 1;
                        self.todo_list.push((result[rlen - ridx - 1], item.1+1));
                    }
                }
            }
        }
        None
    }
}
