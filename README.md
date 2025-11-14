# Rust HTML Rewriter Cloudflare Worker

This project is a Cloudflare Worker written in Rust that acts as a caching proxy to intercept HTML responses from specific domains and inject custom JavaScript snippets into the `<head>` tag. This allows for dynamic modification of HTML content before it's delivered to the client.

## How It Works

The worker intercepts incoming `GET` requests and checks if the URL matches one of the configured domains. If it does, the worker fetches the original HTML content, injects the specified JavaScript snippets, and then returns the modified HTML to the client. The modified response is also cached to improve performance on subsequent requests.

### HTML Rewriting with `lol_html`

The core of the HTML rewriting functionality is handled by the `lol_html` crate, which provides a streaming HTML rewriter that allows for CSS-selector-based modifications.

In this worker, the rewriting is configured to target the `<head>` element. When a matching request is processed, the `HtmlRewriter` is set up with an `element!` handler that appends the required script tags to the head of the document.

The process is as follows:

1.  **Request Interception**: The worker intercepts an incoming request.
2.  **Domain Matching**: It checks the request URL to see if it belongs to one of the target domains.
3.  **Content Fetching**: If the domain matches, the worker fetches the original HTML from the origin server.
4.  **HTML Rewriting**: The `lol_html` rewriter processes the HTML stream and injects the predefined scripts into the `<head>` element.
5.  **Response Caching**: The modified HTML is cached to speed up future requests.
6.  **Response Delivery**: The final HTML is sent to the client.

### Adding or Removing Sites

To add, remove, or modify the scripts for a site, you need to edit the `src/lib.rs` file.

#### Adding a New Site

1.  **Define a Script Constant**: If the new site requires a unique set of scripts, define a new constant array of strings containing the HTML script snippets. For example:

    ```rust
    const NEW_SITE_SCRIPTS: &[&str] = &[
        r#"<script src="..."></script>"#,
        // ... other scripts
    ];
    ```

2.  **Add a Domain Check**: In the `handle_request` function, add a new `else if` condition to the domain matching block:

    ```rust
    // ...
    } else if url_str.contains("newsite.com") {
        log::info!("Injecting to newsite.com");
        Some(NEW_SITE_SCRIPTS)
    // ...
    ```

#### Removing a Site

To remove a site, simply delete its corresponding `else if` block from the `handle_request` function. If the script constants are no longer needed, you can remove them as well to keep the code clean.

### Building the Project

To compile the worker, run the following command:

```bash
cargo build --target wasm32-unknown-unknown
```

This will produce a WebAssembly binary in the `target/wasm32-unknown-unknown/debug` directory, which can then be deployed to Cloudflare Workers.
