# Frontest

<p>
  <img
    href="https://crates.io/crates/frontest"
    alt="Crates.io"
    src="https://img.shields.io/crates/v/frontest" />
  <img
    href="https://docs.rs/frontest/latest/frontest/"
    alt="docs.rs"
    src="https://img.shields.io/docsrs/frontest/latest" />
  <img
    alt="tests status"
    src="https://img.shields.io/github/workflow/status/Zwo1in/frontest-rs/tests"
    href="https://github.com/Zwo1in/frontest-rs" />
  <img
    href="https://crates.io/crates/frontest"
    alt="downloads"
    src="https://img.shields.io/crates/d/frontest" />
  <img
    href="https://crates.io/crates/frontest"
    alt="licenses"
    src="https://img.shields.io/crates/l/frontest" />
</p>

### A lightweight library to query and assert DOM.

Frontest is heavily inspired by [dom-testing-library](https://testing-library.com/docs/dom-testing-library/intro`dom-testing-library) and [react-testing-library](https://testing-library.com/docs/react-testing-library/intro).
It provides a set of queries that you can use to quickly find your elements in document
with respect to accessibility priorities.

### Example:

```rust
use frontest::prelude::*;
use gloo::utils::{body, document};

let div = document().create_element("div").unwrap();
div.set_inner_html(
    r#"<div>
        <label>
            I will start testing my frontend!
            <button>
                Take the red pill
            </button>
        </label>
        <label>
            It's too problematic dude...
            <button>
                Take the blue pill
            </button>
        </label>
    </div>"#,
);
body().append_child(&div).unwrap();

let go_to_matrix = div
    .get(&HasRole("button").and(Not(HasLabel("It's too problematic dude..."))))
    .unwrap();
go_to_matrix.click();

body().remove_child(&div).unwrap();
```

### About testing:

 This library aims to allow developers to test their application in a way that a user would interact with it.
 For this purpose it is recommended to prioritize certain queries above another.
 Currently only two matchers are implemented. More will be available in future releases.
 Matchers should be prioritized like so:
 - `HasRole` Should always be used where possible. It allows accessing elements that are exposed into accessibility tree.
 - `HasLabel` Also should be used where possible. Is supported by screen readers and allows for easier focusing elements.
 - `HasPlaceholder` Not as great option as predecessors, however still a better alternative than `HasText` for accessible elements.
 - `HasText` Can be used to select non-interactive components or further restrict other queries.

### Matchers:

Matchers are predicates for `HtmlElement`. They return `true` if given element suffices some criteria
 or `false` otherwise.

Using the matcher `Not` and methods from `Joinable` trait it is possible to combine multiple matchers into
 a logical expression.

#### You can easily implement your own `Matcher`s.

```rust
struct IsHidden;

impl Matcher for IsHidden {
    fn matches(&self, elem: &HtmlElement) -> bool {
        elem.hidden()
    }
}

let div = document().create_element("div").unwrap();
div.set_inner_html(
    r#"<button hidden>
        Yayyy frontend in rust!
    </button>"#
);
div.append_child(&div).unwrap();

assert!(div.get(&IsHidden).is_some());

body().remove_child(&div).unwrap();
```

### Integration:
Tests should be run using [wasm-bindgen-test](https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/usage.html`wasm-bindgen-test). It allows running them directly in browsers or in node-js.


Currently this crate provides a `render` function that allows for quickly rendering any `html` created with `yew`.
It was choosen to render the html instead of directly taking a component so it is easier to wrap them with `ContextProvider` and so on.

### Example:
```rust
#[function_component(Incrementable)]
fn incrementable() -> Html {
    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        Callback::from(move |_| counter.set(*counter + 1))
    };
    html! {
        <div>
            <p>{ format!("Value: {}", *counter) }</p>
            <button {onclick}>{ "Add" }</button>
        </div>
    }
}

use frontest::prelude::*;
use frontest::yew::render;

#[wasm_bindgen_test]
async fn clicking_on_button_should_increment_value() {
    let mount = render(html! { <Incrementable /> }).await;
    let value = mount.get(&HasText("Value:")).unwrap();
    let button = mount.get(&HasRole("button")).unwrap();

    assert_eq!("Value: 0", value.inner_text());
    button.click();
    assert_eq!("Value: 1", value.inner_text());

    body().remove_child(&mount).unwrap();
}
```

### Warning:

`wasm-bindgen-test` runs all tests sequentially and let them manipulate real DOM.
However it doesn't recreate full DOM for each test, so things done in one test may impact others.
Always make sure you are doing a proper cleanup of DOM after your tests eg. remove mounted child element.
Hopefully in future this library will provide some kind of RAII for running tests.
