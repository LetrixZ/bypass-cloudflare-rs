# Bypass Cloudflare

Library that allows you to get the Cloudflare clearance token and the related User-Agent.\
It uses `headless-chrome` under the hood with the `--disable-blink-features=AutomationControlled` argument passed.

### Usage

The first parameter is the URL and the second is a CSS selector that should be available in the site to ensure that the Cloudflare captcha was passed.

```rust
let params = bypass_cloudflare::get_params("https://nowsecure.nl/#relax", "p.lead")
    .map_err(|err| err.to_string())?;
```

You can also use `bypass_cloudflare::get_params_with_interceptor` with an interceptor to, for example, load only certain resources instead of requesting everything in the page.

```rust
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
```

```rust
let interceptor: Arc<dyn RequestInterceptor + Send + Sync> = Arc::new(MinimalInterceptor {});

let params = bypass_cloudflare::get_params_with_interceptor("https://nowsecure.nl/#relax", "p.lead", interceptor)
    .map_err(|err| err.to_string())?;
```
