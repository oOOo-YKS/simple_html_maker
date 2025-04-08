use std::io::{Result, Error, ErrorKind};
use std::path::PathBuf;
use html_escape::encode_text;
use std::fmt;
use std::collections::HashMap;

/// HTML attribute encoding function
fn encode_attribute(s: &str) -> String {
    encode_text(s).to_string()
}

/// A trait representing any HTML element that can be rendered to a string
pub trait HtmlElement: {
    /// Returns the HTML tag name for this element (e.g., "div", "span", "img")
    /// Empty string means raw HTML content with no wrapping tag
    fn tag(&self) -> &str;
    
    /// Returns the element's attributes as key-value pairs
    /// None means no attributes will be rendered
    fn attributes(&self) -> Option<Vec<(String, String)>>;
    
    /// Returns the element's text content (properly escaped)
    /// None means no text content will be rendered
    fn text(&self) -> Option<String>;
    
    /// Returns a list of child elements
    /// Default implementation returns an empty vector
    fn children(&self) -> Vec<&dyn HtmlElement> {
        Vec::new()
    }
    
    /// Returns true if this is a void/self-closing element (like <img/> or <br/>)
    /// Determines whether the element should be rendered with a closing tag
    fn is_void_element(&self) -> bool;
    
    /// Renders the element to an HTML string
    /// Can be overridden for custom rendering behavior
    fn render(&self) -> String where Self: Sized {
        render_element(self)
    }
    
    /// Helper to check if element has any content (text or children)
    fn has_content(&self) -> bool {
        self.text().is_some() || !self.children().is_empty()
    }
}

/// 基础文本元素
pub struct TextElement {
    content: String,
}

impl TextElement {
    /// 创建新的文本元素
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl HtmlElement for TextElement {
    fn tag(&self) -> &str {
        "span"
    }

    fn attributes(&self) -> Option<Vec<(String, String)>> {
        None
    }

    fn text(&self) -> Option<String> {
        // 基础转义
        let encoded = encode_text(&self.content).to_string();
        // 额外处理单引号（根据 OWASP 推荐）
        let full_encoded = encoded.replace('\'', "&#x27;");
        Some(full_encoded)
    }

    fn is_void_element(&self) -> bool {
        false
    }
}

/// Raw HTML content that won't be escaped
pub struct RawHtml {
    content: String,
}

impl RawHtml {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl HtmlElement for RawHtml {
    fn tag(&self) -> &str {
        ""
    }

    fn attributes(&self) -> Option<Vec<(String, String)>> {
        None
    }

    fn text(&self) -> Option<String> {
        Some(self.content.clone())
    }

    fn is_void_element(&self) -> bool {
        true
    }
    
    fn render(&self) -> String {
        self.content.clone()
    }
}

/// 图像元素
#[derive(Default)]
pub struct ImageElement {
    src: String,
    alt: Option<String>,
    attributes: HashMap<String, String>,
}

impl ImageElement {
    pub fn new(src: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            alt: None,
            attributes: HashMap::new(),
        }
    }

    pub fn with_alt(mut self, alt: impl Into<String>) -> Self {
        self.alt = Some(alt.into());
        self
    }
    
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

impl HtmlElement for ImageElement {
    fn tag(&self) -> &str {
        "img"
    }

    fn attributes(&self) -> Option<Vec<(String, String)>> {
        let mut attrs = vec![("src".to_string(), encode_attribute(&self.src).to_string())];
        if let Some(alt) = &self.alt {
            attrs.push(("alt".to_string(), encode_attribute(alt).to_string()));
        }
        
        // Add any additional attributes
        for (key, value) in &self.attributes {
            attrs.push((key.clone(), encode_attribute(value).to_string()));
        }
        Some(attrs)
    }

    fn text(&self) -> Option<String> {
        None
    }

    fn is_void_element(&self) -> bool {
        true
    }
}

/// 容器元素（可嵌套）
pub struct ContainerElement {
    tag: String,
    attributes: HashMap<String, String>,
    children: Vec<Box<dyn HtmlElement>>,
    classes: Vec<String>,
    id: Option<String>,
}

impl ContainerElement {
    /// 创建新的容器元素
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            attributes: HashMap::new(),
            children: Vec::new(),
            classes: Vec::new(),
            id: None,
        }
    }

    /// 添加HTML属性
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// 添加子元素
    pub fn with_child(mut self, child: impl HtmlElement + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }
    
    /// Add multiple children at once
    pub fn with_children(mut self, children: Vec<Box<dyn HtmlElement>>) -> Self {
        self.children.extend(children);
        self
    }
    
    /// Add a class to the element
    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.classes.push(class.into());
        self
    }
    
    /// Set element ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
    
    /// Add text content to the container
    pub fn with_text(self, text: impl Into<String>) -> Self {
        self.with_child(TextElement::new(text))
    }
}

impl HtmlElement for ContainerElement {
    fn tag(&self) -> &str {
        &self.tag
    }

    fn attributes(&self) -> Option<Vec<(String, String)>> {
        let mut result = Vec::new();
        
        // Add ID if present
        if let Some(id) = &self.id {
            result.push(("id".to_string(), encode_attribute(id).to_string()));
        }
        
        // Add classes if any
        if !self.classes.is_empty() {
            let class_str = self.classes.join(" ");
            result.push(("class".to_string(), encode_attribute(&class_str).to_string()));
        }
        
        // Add all other attributes
        for (key, value) in &self.attributes {
            result.push((
                encode_attribute(key).to_string(),
                encode_attribute(value).to_string(),
            ));
        }
        
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    fn text(&self) -> Option<String> {
        None
    }

    fn children(&self) -> Vec<&dyn HtmlElement> {
        self.children.iter().map(|c| c.as_ref()).collect()
    }

    fn is_void_element(&self) -> bool {
        false
    }
}

impl fmt::Display for ContainerElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}

/// HTML文档构建器
pub struct HtmlDocumentBuilder {
    doctype: String,
    title: Option<String>,
    head_elements: Vec<Box<dyn HtmlElement>>,
    body_elements: Vec<Box<dyn HtmlElement>>,
    lang: Option<String>,
    meta_tags: Vec<(String, String)>,
    stylesheets: Vec<String>,
    scripts: Vec<String>,
    body_attributes: HashMap<String, String>,
}

impl Default for HtmlDocumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HtmlDocumentBuilder {
    /// 初始化构建器
    pub fn new() -> Self {
        Self {
            doctype: "<!DOCTYPE html>".to_string(),
            title: None,
            head_elements: Vec::new(),
            body_elements: Vec::new(),
            lang: Some("en".to_string()),
            meta_tags: vec![("charset".to_string(), "UTF-8".to_string())],
            stylesheets: Vec::new(),
            scripts: Vec::new(),
            body_attributes: HashMap::new(),
        }
    }

    /// 设置文档标题
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 添加head区域元素
    pub fn add_head_element(mut self, element: impl HtmlElement + 'static) -> Self {
        self.head_elements.push(Box::new(element));
        self
    }

    /// 添加body区域元素
    pub fn add_body_element(mut self, element: impl HtmlElement + 'static) -> Self {
        self.body_elements.push(Box::new(element));
        self
    }
    
    /// Set HTML language attribute
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.lang = Some(lang.into());
        self
    }
    
    /// Add a meta tag
    pub fn add_meta(mut self, name: impl Into<String>, content: impl Into<String>) -> Self {
        self.meta_tags.push((name.into(), content.into()));
        self
    }
    
    /// Add a CSS stylesheet link
    pub fn add_stylesheet(mut self, href: impl Into<String>) -> Self {
        self.stylesheets.push(href.into());
        self
    }
    
    /// Add a JavaScript source
    pub fn add_script(mut self, src: impl Into<String>) -> Self {
        self.scripts.push(src.into());
        self
    }
    
    /// Add attribute to body tag
    pub fn add_body_attribute(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.body_attributes.insert(name.into(), value.into());
        self
    }

    /// 构建完整HTML文档
    pub fn build(self) -> String {
        let mut output = String::new();
        
        // 文档类型声明
        output.push_str(&self.doctype);
        
        // Start HTML tag with optional lang attribute
        if let Some(lang) = self.lang {
            output.push_str(&format!("\n<html lang=\"{}\">", encode_attribute(&lang)));
        } else {
            output.push_str("\n<html>");
        }
        output.push('\n');

        // Head部分
        output.push_str("<head>\n");
        
        // Add meta tags
        for (name, content) in &self.meta_tags {
            if name == "charset" {
                output.push_str(&format!("<meta charset=\"{}\">\n", encode_attribute(content)));
            } else {
                output.push_str(&format!(
                    "<meta name=\"{}\" content=\"{}\">\n",
                    encode_attribute(name),
                    encode_attribute(content)
                ));
            }
        }
        
        // Add title if present
        if let Some(title) = &self.title {
            output.push_str(&format!(
                "<title>{}</title>\n",
                encode_text(title).to_string()
            ));
        }
        
        // Add stylesheets
        for href in &self.stylesheets {
            output.push_str(&format!(
                "<link rel=\"stylesheet\" href=\"{}\">\n",
                encode_attribute(href)
            ));
        }
        
        // Add other head elements
        for element in &self.head_elements {
            output.push_str(&render_element(&**element)); // 双重解引用转换为 trait 对象
            output.push('\n');
        }
        output.push_str("</head>\n");

        // Body部分 with optional attributes
        output.push_str("<body");
        
        // Add body attributes if any
        for (name, value) in &self.body_attributes {
            output.push_str(&format!(" {}=\"{}\"", 
                encode_attribute(name), 
                encode_attribute(value)));
        }
        output.push_str(">\n");
        
        // Add body elements
        for element in &self.body_elements {
            output.push_str(&render_element(&**element));
            output.push('\n');
        }
        
        // Add scripts at the end of body
        for src in &self.scripts {
            output.push_str(&format!(
                "<script src=\"{}\"></script>\n",
                encode_attribute(src)
            ));
        }
        
        output.push_str("</body>\n</html>");

        output
    }
}

/// 递归渲染HTML元素
fn render_element(element: &dyn HtmlElement) -> String {
    // Special case for raw HTML
    if element.tag().is_empty() && element.is_void_element() {
        return element.text().unwrap_or_default();
    }
    
    let mut html = String::new();
    
    // 开始标签
    html.push_str(&format!("<{}", element.tag()));
    
    // 添加属性
    if let Some(attrs) = element.attributes() {
        for (key, value) in attrs {
            html.push_str(&format!(" {}=\"{}\"", key, value));
        }
    }
    
    // 处理自闭合标签
    if element.is_void_element() {
        html.push_str(" />");
        return html;
    }
    
    html.push('>');
    
    // 添加文本内容
    if let Some(text) = element.text() {
        html.push_str(&text);
    }
    
    // 递归渲染子元素
    for child in element.children() {
        html.push_str(&render_element(child));
    }
    
    // 闭合标签
    html.push_str(&format!("</{}>", element.tag()));
    
    html
}

/// 保存HTML文档到文件
pub fn save_html(path: impl Into<PathBuf>, content: &str) -> Result<()> {
    let path = path.into();
    
    // Ensure parent directories exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| 
            Error::new(ErrorKind::Other, format!("Failed to create directory: {}", e))
        )?;
    }
    
    std::fs::write(&path, content)
        .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to save HTML to {:?}: {}", path, e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_document() {
        let doc = HtmlDocumentBuilder::new()
            .title("Test Page")
            .add_body_element(TextElement::new("Hello World!"))
            .add_body_element(ImageElement::new("test.jpg").with_alt("Test Image"))
            .build();

        assert!(doc.contains("<title>Test Page</title>"));
        assert!(doc.contains("<span>Hello World!</span>"));
        assert!(doc.contains("<img src=\"test.jpg\" alt=\"Test Image\" />"));
    }

    #[test]
fn test_xss_protection() {
    let malicious = "<script>alert('hack')</script>";
    let doc = TextElement::new(malicious).text().unwrap();
    assert_eq!(doc, "&lt;script&gt;alert(&#x27;hack&#x27;)&lt;/script&gt;");
}

    #[test]
    fn test_nested_elements() {
        let container = ContainerElement::new("div")
            .with_attribute("class", "container")
            .with_child(TextElement::new("Nested Content"))
            .with_child(ContainerElement::new("p").with_child(TextElement::new("Deep")));

        let rendered = render_element(&container);
        assert!(rendered.contains("<div class=\"container\">"));
        assert!(rendered.contains("<span>Nested Content</span>"));
        assert!(rendered.contains("<p><span>Deep</span></p>"));
    }
    
    #[test]
    fn test_raw_html() {
        let html = RawHtml::new("<b>Bold</b> text");
        assert_eq!(html.render(), "<b>Bold</b> text");
    }
    
    #[test]
    fn test_container_with_classes() {
        let div = ContainerElement::new("div")
            .with_class("primary")
            .with_class("large")
            .with_id("main-content");
            
        let rendered = div.render();
        assert!(rendered.contains("id=\"main-content\""));
        assert!(rendered.contains("class=\"primary large\""));
    }
}