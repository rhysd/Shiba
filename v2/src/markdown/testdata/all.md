![icon](../../../assets/icon.iconset/icon_64x64.png) Markdown rendering test
===================================================================

This document renders all elements of Markdown text (including extensions by GitHub) for testing.

<a id="top"></a>

# H1

## H2

### H3

#### H4

##### H5

###### H6

Sext Heading H1
===============

Sext Heading H2
---------------

# Code blocks

Indented code block:

    a sinmple
    code block

Fenced code block

```
let world = "world";

fn main() {
    println!("Hello, {}", world);
}
```

Highlighted code block:

```rust
let world = "world";

fn main() {
    println!("Hello, {}", world);
}
```

# HTML block

A table using `<table>` tag and friends:

<table><tr><td>
Hello
</td><td>
world
</td></tr></table>

The `<script>` tag should be stripped out:

<script>
alert("hey!");
</script>

The `<details>` tag can fold contents:

<details>
<summary>Folded!</summary>
<pre>
Lorem Ipsum is simply dummy text of the printing
and typesetting industry. Lorem Ipsum has been the
industrial standard dummy text ever since the 1500s,
when an unknown printer took a galley of type and
scrambled it to make a type specimen book.
</pre>
</details>

# Link references

[minimal absolute link]

[minimal relative link]

[abosolute link with title]

# Paragraphs

Single line paragraph.

Multiple lines paragraph.
Multiple lines paragraph.
Multiple lines paragraph.

Paragraph containing br.  
Paragraph containing br.

# Block quotes

> Simple single-line block quote.

> Simple multiple-lines block quote.  
> Simple multiple-lines block quote.  
> Simple multiple-lines block quote.

> ## Nested title
>
> Nested paragraph. Nested [link][foo].
>
>
> ```rust
> println!("nested fenced code block!");
> ```
>
> [foo]: https://example.com

>>> Multiple
>>> levels of
>>> block quote

<!-- ` This comment fixes broken syntax highlight by Vim -->

[minimal absolute link]: https://example.com
[minimal relative link]: ../../../README.md
[abosolute link with title]: https://example.com "this is title"

# Lists

Ordered list:

1. One
   1) One-One
   2) One-Two
      1. One-Two-One
      2. One-Two-Two
2. Two
   1. Two-One

Ordered list starting with 10:

10. Ten
11. Eleven
12. Eleven

Unordered list:

* One
   - One-One
   - One-Two
      + One-Two-One
      + One-Two-Two
* Two
   + Two-One

Mixed nested list:

- One
  1. One-One
  2. One-Two
     * One-Two-Three
     * One-Two-Four
  3. One-Three
- Two
  + Two-Three
- Three

Nested paragraph in list:

- This is sentence.
  Paragraph continues.  
  This starts new line.
  > Nested block quote  
  > Nested block quote
  ```rust
  println!("nested code block in list item");
  ```
  - Second level list item.
    Paragraph continues.  
    This starts new line.
    > Nested block quote  
    > Nested block quote
    ```rust
    println!("nested code block in list item");
    ```

# Inline codes

This `sentence` contains ``inline code``.

# Emphasis

This *is* a _sentence_ containing **emphasized text** for __testing__.

# Links

- [external link](https://example.com)
- [external link with title](https://example.com "this is title")
- [internal link](../../../README.md)
- [internal link with title](../../../README.md "this is title")
- [fragment link](#top)
- [`link` _containing_ **inlines**](https://example.com)

# Images

![simple image](../../../assets/icon.iconset/icon_64x64.png)
![image with title](../../../assets/icon.iconset/icon_64x64.png "this is title!")
![external image](https://github.com/rhysd/Shiba/blob/main/v2/assets/icon.iconset/icon_64x64.png?raw=true)
[![image link](../../../assets/icon.iconset/icon_64x64.png)](https://example.com "image link")

# Auto links

Standard auto link: <https://example.com>

Raw URL auto link extension: https://example.com

# Inline HTML

This is <a href="https://example.com">a link with anchor tag</a> in a paragraph block.

This is nested <code>inline, <a href="https://example.com">HTML</a></code> items.

This inline script tag <script>alert('hello')</script> should be sanitized.

Code block using `<pre><code>` is written in inline HTML but rendered as block:

<pre><code>This is
code block</code></pre>

# Task lists extension

Task lists:

- [x] Checked
- [ ] Unchecked

Nested task lists:

- [x] Checked
  - [ ] Unchecked
- [ ] Unchecked
  - [x] Checked

# Tables extension

Simple table:

| foo | bar |
| --- | --- |
| aaa | bbb |
| ccc | ddd |

Aligned table

| left | center | right |
|:--- |:---:| ---:|
| A | B | C |

# Strikethrough extension

~This line is deleted.~

~~These lines  
are deleted.~~

# Emoji extension

:dog: :cat: :emoji-does-not-exist: :+1: :-1:

# Foot notes extension

Here is a simple footnote[^1]. With named label[^label].

[^1]: My reference.

[^label]: This is note with label

# Math extension

Inline formula: $e = mc^2$.

Display formula:

$$\left( \sum_{k=1}^n a_k b_k \right)^2 \leq \left( \sum_{k=1}^n a_k^2 \right) \left( \sum_{k=1}^n b_k^2 \right)$$

Formula block fenced with `math`:

```math
\left( \sum_{k=1}^n a_k b_k \right)^2 \leq \left( \sum_{k=1}^n a_k^2 \right) \left( \sum_{k=1}^n b_k^2 \right)
```

# Mermaid extension

Simple diagram using [mermaid.js](https://github.com/mermaid-js/mermaid):

```mermaid
graph TD;
    A-->B;
    A-->C;
    B-->D;
    C-->D;
```

# Alerts extension

Alert notations in block quote style.

> [!NOTE]
> Highlights information that users should take into account, even when skimming.

> [!TIP]
> Optional information to help a user be more successful.

> [!IMPORTANT]
> Crucial information necessary for users to succeed.

> [!WARNING]
> Critical content demanding immediate user attention due to potential risks.

> [!CAUTION]
> Negative potential consequences of an action.

Resources:

- The specification is described in this discussion thread: https://github.com/orgs/community/discussions/16925
- Official document: https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax#alerts
- Changelog: https://github.blog/changelog/2023-12-14-new-markdown-extension-alerts-provide-distinctive-styling-for-significant-content/
