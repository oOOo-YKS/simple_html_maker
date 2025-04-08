# Simple HTML Generator for Rust

一个安全、轻量的HTML文档生成库，提供类型安全的元素构建和自动XSS防护。

## 特性

- 🛡️ 自动HTML特殊字符转义（支持文本和属性值）
- 📦 内置基础元素：文本、图片、容器、原始HTML
- 🌳 支持无限嵌套的文档结构
- 📝 流畅的构建器模式API
- ✅ 符合HTML5标准输出
- 🧪 完整测试覆盖（包括XSS防护测试）

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
html_escape = "0.2"
simple_html_maker = { git = "https://github.com/yourusername/simple_html_maker" }
```

## 快速开始

```rust
use simple_html_maker::html_file::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建文档
    let doc = HtmlDocumentBuilder::new()
        .title("My Page")
        .add_body_element(
            ContainerElement::new("div")
                .with_id("main")
                .with_class("content")
                .with_text("Hello World!")
                .with_child(
                    ImageElement::new("cat.jpg")
                        .with_alt("Cute Cat")
                )
        )
        .build();

    // 保存到文件
    save_html("index.html", &doc)?;
    Ok(())
}
```

## 基本用法

### 创建文本元素
```rust
let text = TextElement::new("<script>alert('xss')</script>");
// 输出: <span>&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;</span>
```

### 构建嵌套结构
```rust
let menu = ContainerElement::new("ul")
    .with_child(
        ContainerElement::new("li")
            .with_class("item")
            .with_text("Home")
    )
    .with_child(
        ContainerElement::new("li")
            .with_class("item")
            .with_text("About")
    );
```

### 添加图片
```rust
let img = ImageElement::new("photo.png")
    .with_attribute("width", "300")
    .with_alt("My Photo");
// 输出: <img src="photo.png" alt="My Photo" width="300" />
```

## 安全警告

⚠️ 使用 `RawHtml` 时需特别注意：
```rust
// 不安全用法（慎用！）
let dangerous = RawHtml::new("<script>untrusted_content</script>");
```
- 仅用于可信内容
- 不要直接渲染用户输入
- 优先使用安全元素（TextElement/ImageElement等）

## 贡献

欢迎提交 issue 和 PR：
1. Fork 仓库
2. 创建特性分支 (`git checkout -b feature/awesome`)
3. 提交更改 (`git commit -am 'Add awesome feature'`)
4. 推送分支 (`git push origin feature/awesome`)
5. 创建 Pull Request

## 许可证

MIT License © 2024 oOOo-YKS
