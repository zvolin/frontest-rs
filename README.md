Quick and lightweight library to query and assert DOM.

Frontest is heavily inspired by [`dom-testing-library`] and [`react-testing-library`].
It provides a set of queries that you can use to quickly find your elements in document
with respect to accessibility priorities.

# Basic usage:

Let's write a test for a simple [`yew`] component that displays it's value and increments it on button click.
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

use frontest::{Query, HasText, HasRole};
use frontest::yew::render;
use yew::html;

#[wasm_bindgen_test]
async fn clicking_on_button_should_increment_value() {
    let mount = render(html! { <Incrementable /> }).await;
    let value = mount.get(&HasText("Value:")).unwrap();
    let button = mount.get(&HasRole("button")).unwrap();

    assert_eq!("Value: 0", value.inner_text());
    button.click();
    assert_eq!("Value: 1", value.inner_text());
}
```

# About testing:

This library aims to allow developers to test their application in a way that a user would interact with it.
For this purpose it is recommended to prioritize certain queries above another.
Currently only two matchers are implemented. More will be available in future releases.
Matchers should be prioritized like so:
- [`HasRole`] Should always be used where possible. It allows accessing elements that are exposed into accessibility tree.
- [`HasText`] Can be used to select non-interactive components or further restrict other queries.

# Matchers:

Matchers are predicates for [`HtmlElement`]. They return [`true`] if given element suffices some criteria
or [`false`] otherwise.

Using the matcher [`Not`] and methods from [`Joinable`] trait it is possible to combine multiple matchers into
a logical expression.

# Integration:
Tests should be run using [`wasm-bindgen-test`]. It allows running them directly in browsers or in node-js.

Currently this crate provides a [`render`] function that allows for quickly rendering any [`html`] created with [`yew`].
It was choosen to render the html instead of directly taking a component so it is easier to wrap them with [`ContextProvider`] and so on.

## Example:
```rust
use frontest::yew::render;
use yew::prelude::*;

#[wasm_bindgen_test]
async fn foo() {
    let elem = render(html! {
        <ContextProvider<Bar> context={Bar {}}>
            <Baz />
        </ContextProvider<Bar>>
    }).await;
}
```

