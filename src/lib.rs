use std::{error::Error, ffi::OsStr, sync::Arc};

use headless_chrome::{browser::tab::RequestInterceptor, Browser, LaunchOptions};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Params {
    pub token: Option<String>,
    pub user_agent: Option<String>,
}

fn browse(
    url: &str,
    element_selector: &str,
    interceptor: Option<Arc<dyn RequestInterceptor + Send + Sync>>,
) -> Result<Params, Box<dyn std::error::Error>> {
    let browser = Browser::new(LaunchOptions {
        headless: false,
        args: vec![OsStr::new("--disable-blink-features=AutomationControlled")],
        ..Default::default()
    })?;

    let tab = browser.new_tab()?;

    if let Some(interceptor) = interceptor {
        tab.enable_request_interception(interceptor)?;
        tab.enable_fetch(None, None)?;
    }

    tab.navigate_to(url)?;
    tab.wait_for_element(element_selector)?;

    let params = Params {
        token: tab
            .get_cookies()?
            .into_iter()
            .find(|cookie| cookie.name.eq("cf_clearance"))
            .map(|cookie| cookie.value),
        user_agent: Some(browser.get_version()?.user_agent),
    };

    Ok(params)
}

pub fn get_params(url: &str, element_selector: &str) -> Result<Params, Box<dyn Error>> {
    browse(url, element_selector, None)
}

pub fn get_params_with_interceptor(
    url: &str,
    element_selector: &str,
    interceptor: Arc<dyn RequestInterceptor + Send + Sync>,
) -> Result<Params, Box<dyn Error>> {
    browse(url, element_selector, Some(interceptor))
}

#[cfg(test)]
mod tests {
    use headless_chrome::{
        browser::{
            tab::RequestPausedDecision,
            transport::{SessionId, Transport},
        },
        protocol::cdp::{
            Fetch::{events::RequestPausedEvent, FailRequest},
            Network::{self, ResourceType},
        },
    };

    use super::*;

    #[test]
    fn cf_params() {
        let result =
            get_params("https://nowsecure.nl/#relax", "p.lead").expect("Failed to get params");

        assert!(result.token.is_some(), "Token is empty");
        assert!(result.user_agent.is_some(), "Token is empty");
    }

    struct MinimalInterceptor {}

    impl RequestInterceptor for MinimalInterceptor {
        fn intercept(
            &self,
            _transport: Arc<Transport>,
            _session_id: SessionId,
            event: RequestPausedEvent,
        ) -> RequestPausedDecision {
            match event.params.resource_Type {
                ResourceType::Document | ResourceType::Script | ResourceType::Xhr => {
                    RequestPausedDecision::Continue(None)
                }
                _ => RequestPausedDecision::Fail(FailRequest {
                    error_reason: Network::ErrorReason::BlockedByClient,
                    request_id: event.params.request_id,
                }),
            }
        }
    }

    #[test]
    fn cf_params_interceptor() {
        let interceptor: Arc<dyn RequestInterceptor + Send + Sync> =
            Arc::new(MinimalInterceptor {});

        let result =
            get_params_with_interceptor("https://nowsecure.nl/#relax", "p.lead", interceptor)
                .expect("Failed to get params");

        assert!(result.token.is_some(), "Token is empty");
        assert!(result.user_agent.is_some(), "Token is empty");
    }
}
