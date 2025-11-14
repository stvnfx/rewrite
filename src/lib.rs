use log;

// --- LOL_HTML IMPORTS (Correct for v1.1.0) ---
use lol_html::{html_content::ContentType, element, HtmlRewriter, Settings};
// NO ElementContentHandler trait import is needed!

// --- WORKER IMPORTS (These are correct) ---
use worker::{
    event, Cache, Context, Env, Fetch, Method, Request, Response, Result,
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

const NETWRK13_SCRIPTS: &[&str] = &[
    r#"<script defer src="https://um.netwrk13.dev/script.js" data-website-id="8533c887-958d-4e02-b3b8-9afac502d1bc"></script>"#
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
    }
    .await;

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

    if let Some(response) = cache.get(&cache_key, false).await? {
        log::info!("Cache hit for: {}", url_str);
        return Ok(response);
    }

    log::info!("Cache miss for: {}. Fetching and caching.", url_str);
    let mut response = Fetch::Request(req.clone()?).send().await?;

    // --- REWRITER LOGIC ---
    let scripts_to_inject: Option<&[&str]> = if url_str.contains("mpshecosystem.com") {
        log::info!("Injecting to mpshecosystem.com");
        Some(MPSHECOSYSTEM_SCRIPTS)
    } else if url_str.contains("mpshsoftware.com") {
        log::info!("Injecting to mpshsoftware.com");
        Some(MPSHSOFTWARE_SCRIPTS)
    } else if url_str.contains("netwrk13.dev") {
        log::info!("Injecting to netwrk13.dev");
        Some(NETWRK13_SCRIPTS)
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

    let (body, headers) = if let Some(scripts) = scripts_to_inject {
        let mut output = Vec::new();

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
                output.extend_from_slice(chunk);
            },
        );

        let original_headers = response.headers().clone();
        let body_bytes = response.bytes().await?;

        rewriter
            .write(&body_bytes)
            .map_err(|e| worker::Error::from(e.to_string()))?;
        rewriter
            .end()
            .map_err(|e| worker::Error::from(e.to_string()))?;

        (output, original_headers)
    } else {
        let headers = response.headers().clone();
        let body = response.bytes().await?;
        (body, headers)
    };

    let mut response_for_client = Response::from_bytes(body.clone())?;
    *response_for_client.headers_mut() = headers.clone();
    response_for_client.headers_mut().set("Cache-Control", "s-maxage=10")?;

    ctx.wait_until(async move {
        let mut response_for_cache = match Response::from_bytes(body) {
            Ok(res) => res,
            Err(e) => {
                log::error!("Failed to create response for caching: {}", e.to_string());
                return;
            }
        };
        *response_for_cache.headers_mut() = headers;
        if let Err(e) = response_for_cache.headers_mut().set("Cache-Control", "s-maxage=10") {
            log::error!("Failed to set cache control for caching response: {}", e.to_string());
            return;
        }

        let cache = Cache::default();
        match cache.put(&cache_key, response_for_cache).await {
            Ok(_) => log::info!("Cached response for: {}", url_str),
            Err(e) => log::error!("Failed to cache response: {}", e.to_string()),
        }
    });

    Ok(response_for_client)
}