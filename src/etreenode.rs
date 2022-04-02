/// Element tree node
///
/// `etree.ETreeNode` stores information of a tree node.
///
/// - `namespace`: value of xmlns or xmlns:XXX
/// - `namespace_abbrev`: none or xmlns:`XXX`
/// - `tag`: {`namespace`}+`localname`
/// - `name`: `namespace_abbrev` + `:` + `localname`
/// - `localname`: tag name
/// - `text`: text between open tag and the next open tag or close tag
/// - `tail`: text between close tag and the next open tag or close tag
/// - `attr`: key-value pairs in the open tag
/// - `idx`: id for the node for internal useage
/// - `route`: descendant route from root to parent for internal usage (format: `#root_idx#child_idx#child_child_idx#`)
///
/// For the following xml file:
/// ```xml
/// <?xml version="1.0" encoding="UTF-8"?>
/// <beans xmlns="http://www.springframework.org/schema/beans"
///        xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
///        xmlns:context="http://www.springframework.org/schema/context"
///        xmlns:mvc="http://www.springframework.org/schema/mvc"
///        xsi:schemaLocation="http://www.springframework.org/schema/beans 
///                            http://www.springframework.org/schema/beans/spring-beans.xsd
///                            http://www.springframework.org/schema/context 
///                            http://www.springframework.org/schema/context/spring-context.xsd
///                            http://www.springframework.org/schema/mvc
///                            http://www.springframework.org/schema/mvc/spring-mvc.xsd">
///     <context:component-scan base-package="xxx.xxx.controller"></context:component-scan>
///     <context:annotation-config />
///     <mvc:default-servlet-handler/>
///     <mvc:annotation-driven/>
///     <mvc:resources mapping="/images/**" location="/images/" />
///     <bean id="xxx" class="xxx.xxx.xxx.Xxx">
///         <property name="xxx" value="xxxx"/>
///     </bean>
/// </beans>
/// ```
/// For node `<context:component-scan base-package="xxx.xxx.controller"></context:component-scan>`:
/// - `namespace`: `"http://www.springframework.org/schema/context"`
/// - `namespace_abbrev`: `"context"`
/// - `tag`: `"http://www.springframework.org/schema/contextcomponent-scan"`
/// - `name`: `"context:component-scan"`
/// - `localname`: `"component-scan"`
/// - `text`: `""`
/// - `tail`: `"\n    "`
/// - `attr`: `[("base-package", "xxx.xxx.controller"), ]`
///
#[derive(Debug, Clone)]
pub struct ETreeNode {
    idx:usize,
    ns:String,
    ns_abbrev:String,
    local_name:String,
    attr:Vec<(String, String)>,
    text:Option<String>,
    tail:String,
    route:String,
}

impl ETreeNode {
    #[allow(dead_code)]
    pub fn new(localname:&str) -> ETreeNode {
        ETreeNode {
            idx:0,
            ns:"".to_string(),
            ns_abbrev:"".to_string(),
            local_name:String::from(localname),
            attr:Vec::new(),
            text:None,
            tail:"".to_string(),
            route:"".to_string(),
        }
    }
    #[allow(dead_code)]
    pub fn get_idx(&self) -> usize {
        self.idx
    }
    #[allow(dead_code)]
    pub fn get_route(&self) -> String {
        self.route.clone()
    }
    #[allow(dead_code)]
    pub fn get_namespace(&self) -> String {
        self.ns.clone()
    }
    #[allow(dead_code)]
    pub fn get_namespace_abbrev(&self) -> String {
        self.ns_abbrev.clone()
    }
    #[allow(dead_code)]
    pub fn get_tag(&self) -> String {
        format!("{{{}}}{}", self.ns, self.local_name)
    }
    #[allow(dead_code)]
    pub fn get_name(&self) -> String {
        if self.ns_abbrev == "" {
            format!("{}", self.local_name)
        } else {
            format!("{}:{}", self.ns_abbrev, self.local_name)
        }
    }
    #[allow(dead_code)]
    pub fn get_localname(&self) -> String {
        format!("{}", self.local_name)
    }
    #[allow(dead_code)]
    pub fn get_text(&self) -> Option<String> {
        self.text.clone()
    }
    #[allow(dead_code)]
    pub fn get_tail(&self) -> String {
        self.tail.clone()
    }
    #[allow(dead_code)]
    pub fn set_idx(&mut self, idx:usize) {
        self.idx = idx;
    }
    #[allow(dead_code)]
    pub fn set_route(&mut self, text:&str) {
        self.route = String::from(text);
    }
    #[allow(dead_code)]
    pub fn set_namespace(&mut self, text:&str) {
        self.ns = String::from(text);
    }
    #[allow(dead_code)]
    pub fn set_namespace_abbrev(&mut self, text:&str) {
        self.ns_abbrev = String::from(text);
    }
    #[allow(dead_code)]
    pub fn set_text(&mut self, text:&str) {
        self.text = Some(String::from(text));
    }
    #[allow(dead_code)]
    pub fn set_tail(&mut self, text:&str) {
        self.tail = String::from(text);
    }
    #[allow(dead_code)]
    pub fn get_attr_count(&self) -> usize {
        self.attr.len()
    }
    #[allow(dead_code)]
    pub fn get_attr_iter(&self) -> std::slice::Iter<(String, String)> {
        self.attr.iter()
    }
    #[allow(dead_code)]
    pub fn get_attr(&self, key:&str) -> Option<String> {
        self.find_attr(key).and_then(|idx| Some(self.attr[idx].1.clone()))
    }
    #[allow(dead_code)]
    pub fn set_attr(&mut self, key:&str, value:&str) -> usize {
        if let Some(idx) = self.find_attr(key) {
            self.attr[idx].1 = String::from(value);
            idx
        } else {
            self.attr.push((String::from(key), String::from(value)));
            self.attr.len()
        }
    }
    fn find_attr(&self, key:&str) -> Option<usize> {
        for i in 0..self.attr.len() {
            if self.attr[i].0 == key {
                return Some(i);
            }
        }
        return None;
    }
}

impl std::fmt::Display for ETreeNode {
    fn fmt(&self, f:&mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{{{}}}{}[", self.ns, self.local_name)?;
        let mut attrs:Vec<String> = Vec::new();
        for item in self.attr.iter() {
            attrs.push(format!("{}=\"{}\"", &item.0, &item.1));
        }
        write!(f, "{}]={:?}", attrs.join(" "), self.text)
    }
}
