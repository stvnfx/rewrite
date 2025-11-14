use log;
use std::cell::RefCell;

// --- LOL_HTML IMPORTS (Correct for v1.1.0) ---
use lol_html::{
    HtmlRewriter,
    Settings,
    element // This is for the element! macro
};
use lol_html::html_content::{
    ContentType
    // Element (Removed, it was unused)
};
// NO ElementContentHandler trait import is needed!

// --- WORKER IMPORTS (These are correct) ---
use worker::{
    event,
    Cache,
    Context,
    Env,
    Fetch,
    Method,
    Request,
    Response,
    Result
};

// --- Shared Script Constants ---
const POPUP_SMART_SCRIPT: &str = r#"<script src="https://cdn.popupsmart.com/bundle.js" data-id="216989" async defer></script>"#;
const GLOW_COOKIES_SCRIPT: &str = r#"<script src="https://cdn.jsdelivr.net/gh/manucaralmo/GlowCookies@3.1.6/src/glowCookies.min.js"></script>"#;
const GLOW_COOKIES_INIT: &str = r#"<script>glowCookies.start('en', {style: 2, 
    hideAfterClick: true,
    bannerLinkText: '  ',
    acceptBtnText: 'Accept',
    bannerDescription: 'We use our own and third-party cookies to personalize content and to analyze web traffic. Read more about our <a href="https://mpsh.fra1.cdn.digitaloceanspaces.com/policy/CookiePolicy.html" target="_blank">Cookie Policy</a> and <a href="https://mpsh.fra1.cdn.digitaloceanspaces.com/policy/PrivacyNotice.html" target="_blank">Privacy Policy.</a>'});</script>"#;
const GLOW_COOKIES_INIT_TIMEOUT: &str = r#"<script>
    setTimeout(function() {      
      glowCookies.start('en', {style: 2, 
    hideAfterClick: true,
    bannerLinkText: '  ',
    acceptBtnText: 'Accept',
    bannerDescription: 'We use our own and third-party cookies to personalize content and to analyze web traffic. Read more about our <a href="https://mpsh.fra1.cdn.digitaloceanspaces.com/policy/CookiePolicy.html" target="_blank">Cookie Policy</a> and <a href="https://mpsh.fra1.cdn.digitaloceanspaces.com/policy/PrivacyNotice.html" target="_blank">Privacy Policy.</a>'});
    }, 1000);
    </script>"#;


// Define the script arrays for each domain
const MPSHSOFTWARE_SCRIPTS: &[&str] = &[
    POPUP_SMART_SCRIPT,
    r#"<script defer data-domain="mpshsoftware.com" data-api="https://mpshsoftware.com/psb/api/event" src="https://mpshsoftware.com/psb/js/psb.js"></script>"#,
    r#"<script data-domain="mpshsoftware.com" data-url="https://mpshsoftware.com/psb/api/event" src="https://mpsh.fra1.cdn.digitaloceanspaces.com/singularity/singularity.js"></script>"#,
    GLOW_COOKIES_SCRIPT,
    GLOW_COOKIES_INIT,
];

const MPSHECOSYSTEM_SCRIPTS: &[&str] = &[
    POPUP_SMART_SCRIPT,
    r#"<script defer data-domain="mpshecosystem.com" data-api="https://mpshecosystem.com/psb/api/event" src="https://mpshecosystem.com/psb/js/psb.js"></script>"#,
    r#"<script data-domain="mpshecosystem.com" data-url="https://mpshecosystem.com/psb/api/event" src="https://mpsh.fra1.cdn.digitaloceanspaces.com/singularity/singularity.js"></script>"#,
    GLOW_COOKIES_SCRIPT,
    GLOW_COOKIES_INIT,
];

const HOMEMARKETPLACE_SCRIPTS: &[&str] = &[
    r#"<script defer data-domain="home.marketplacesuperheroes.com" data-api="https://mpshecosystem.com/psb/api/event" src="https://mpshecosystem.com/psb/js/psb.js"></script>"#,
    GLOW_COOKIES_SCRIPT,
    GLOW_COOKIES_INIT_TIMEOUT,
];

const FOURSPRODUCT_SCRIPTS: &[&str] = &[
    r#"<script defer data-domain="4sproductgauntlet.com" data-api="https://mpshecosystem.com/psb/api/event" src="https://mpshecosystem.com/psb/js/psb.js"></script>"#,
    r#"<script data-domain="4sproductgauntlet.com" data-url="https://mpshecosystem.com/psb/api/event" src="https://mpsh.fra1.cdn.digitaloceanspaces.com/singularity/singularity.js"></script>"#,
    GLOW_COOKIES_SCRIPT,
    GLOW_COOKIES_INIT,
];

const UNIVERSITY_SCRIPTS: &[&str] = &[
    r#"<script defer data-domain="thesuperherouniversity.com" data-api="https://mpshecosystem.com/psb/api/event" src="https://mpshecosystem.com/psb/js/psb.js"></script>"#,
    r#"<script data-domain="thesuperherouniversity.com" data-url="https://mpshecosystem.com/psb/api/event" src="https://mpsh.fra1.cdn.digitaloceanspaces.com/singularity/singularity.js"></script>"#,
    GLOW_COOKIES_SCRIPT,
    GLOW_COOKIES_INIT,
];

#[event(start)]
pub fn start() {
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");
}

#[event(fetch)]
pub async fn main(req: Request, _env: Env, ctx: Context) -> Result<Response> {
    let result = async {
        if req.method() != Method::Get {
            return Fetch::Request(req).send().await;
        }
        handle_request(req, ctx).await
    }.await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Response::error(format!("Error thrown: {}", e), 500),
    }
}

async fn handle_request(req: Request, ctx: Context) -> Result<Response> {
    let cache = Cache::default();
    let cache_key = req.clone()?;
    let url = req.url()?; // Get URL object
    let url_str = url.to_string(); // Get URL as string

    // --- FIX 1: 'match_request' is just 'match' ---
    if let Some(response) = cache.match(&cache_key).await? {
        log::info!("Cache hit for: {}", url_str);
        return Ok(response);
    }

    log::info!("Cache miss for: {}. Fetching and caching.", url_str);
    let response = Fetch::Request(req.clone()?).send().await?;

    // --- REWRITER LOGIC ---
    let scripts_to_inject: Option<&[&str]> = if url_str.contains("mpshecosystem.com") {
        log::info!("Injecting to mpshecosystem.com");
        Some(MPSHECOSYSTEM_SCRIPTS)
    } else if url_str.contains("mpshsoftware.com") {
        log::info!("Injecting to mpshsoftware.com");
        Some(MPSHSOFTWARE_SCRIPTS)
    } else if url_str.contains("home.marketplacesuperheroes.com") {
        log::info!("Injecting to marketplacesuperheroes.com");
        Some(HOMEMARKETPLACE_SCRIPTS)
    } else if url_str.contains("4sproductgauntlet.com") {
        log::info!("Injecting to 4sproductgauntlet.com");
        Some(FOURSPRODUCT_SCRIPTS)
    } else if url_str.contains("thesuperherouniversity.com") {
        log::info!("Injecting to thesuperherouniversity.com");
        Some(UNIVERSITY_SCRIPTS)
    } else {
        None
    };

    let final_response = if let Some(scripts) = scripts_to_inject {
        let output = RefCell::new(Vec::new());

        let element_handler = element!("head", |el| {
            for script in scripts {
                el.append(script, ContentType::Html);
            }
            Ok(())
        });

        let mut rewriter = HtmlRewriter::new(
            Settings {
                element_content_handlers: vec![element_handler],
                ..Settings::default()
            },
            |chunk: &[u8]| {
                output.borrow_mut().extend_from_slice(chunk);
            }
        );

        let body_bytes = response.bytes().await?;

        // --- FIX 2: Handle RewritingError ---
        // We must map the error from lol_html to a worker::Error
        rewriter.write(&body_bytes)
            .map_err(|e| worker::Error::from(e.to_string()))?;
        rewriter.end()
            .map_err(|e| worker::Error::from(e.to_string()))?;

        let body = output.into_inner();
        let mut new_response = Response::from_bytes(body)?;

        // --- FIX 3: headers_mut() doesn't return a Result ---
        // So we remove the '?' after it.
        // And clone_from() returns (), so remove the '?' after it too.
        new_response.headers_mut().clone_from(response.headers());
        new_response.headers_mut().set("Cache-Control", "s-maxage=10")?; // .set() DOES return Result
        
        new_response
    } else {
        let mut new_response = response.cloned()?;
        // --- FIX 3 (Again): headers_mut() doesn't return a Result ---
        new_response.headers_mut().set("Cache-Control", "s-maxage=10")?;
        new_response
    };

    // --- FIX 4 & 5: Handle errors inside wait_until ---
    ctx.wait_until(async move {
        // .cloned() returns a Result, so we must handle it
        match final_response.cloned() {
            Ok(cloned_response) => {
                let cache = Cache::default();
                // 'put_request' is just 'put'
                match cache.put(&cache_key, cloned_response).await {
                    Ok(_) => log::info!("Cached response for: {}", url_str),
                    Err(e) => log::error!("Failed to cache response: {}", e.to_string()),
                }
            }
            Err(e) => {
                log::error!("Failed to clone response for caching: {}", e.to_string());
            }
        }
    });

    Ok(final_response)
}