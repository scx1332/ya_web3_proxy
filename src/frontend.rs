use actix_web::{HttpRequest, Responder, Scope, web};
use actix_web::HttpResponse;
use base64::{Engine as _, alphabet, engine::{self, general_purpose}};

pub async fn fn1() -> impl Responder {
    let base64 = "PCFET0NUWVBFIGh0bWw+CjxodG1sIGxhbmc9ImVuIj4KICAgIDxoZWFkPgogICAgICAgIDxtZXRhIGNoYXJzZXQ9InV0Zi04IiAvPgogICAgICAgIDx0aXRsZT5WaXRlIHRlbXBsYXRlPC90aXRsZT4KICAgICAgICA8bGluayByZWw9Imljb24iIGhyZWY9Ii9mcm9udGVuZC9sb2dvLnBuZyIgLz4KICAgICAgICA8bWV0YSBuYW1lPSJ2aWV3cG9ydCIgY29udGVudD0id2lkdGg9ZGV2aWNlLXdpZHRoLCBpbml0aWFsLXNjYWxlPTEuMCIgLz4KICAgICAgICA8c2NyaXB0IHR5cGU9Im1vZHVsZSIgY3Jvc3NvcmlnaW4gc3JjPSIvZnJvbnRlbmQvYXNzZXRzL2luZGV4LWM1ZDQzNDViLmpzIj48L3NjcmlwdD4KICAgICAgICA8bGluayByZWw9Im1vZHVsZXByZWxvYWQiIGNyb3Nzb3JpZ2luIGhyZWY9Ii9mcm9udGVuZC9hc3NldHMvcmVhY3QtYTEwMjk3MGEuanMiIC8+CiAgICAgICAgPGxpbmsgcmVsPSJtb2R1bGVwcmVsb2FkIiBjcm9zc29yaWdpbiBocmVmPSIvZnJvbnRlbmQvYXNzZXRzL3NjaGVkdWxlci0wNGNlMDU4Mi5qcyIgLz4KICAgICAgICA8bGluayByZWw9Im1vZHVsZXByZWxvYWQiIGNyb3Nzb3JpZ2luIGhyZWY9Ii9mcm9udGVuZC9hc3NldHMvcmVhY3QtZG9tLTFiYWUxMWFkLmpzIiAvPgogICAgICAgIDxsaW5rIHJlbD0ibW9kdWxlcHJlbG9hZCIgY3Jvc3NvcmlnaW4gaHJlZj0iL2Zyb250ZW5kL2Fzc2V0cy9AcmVtaXgtcnVuLWNiNTg0ZGVlLmpzIiAvPgogICAgICAgIDxsaW5rIHJlbD0ibW9kdWxlcHJlbG9hZCIgY3Jvc3NvcmlnaW4gaHJlZj0iL2Zyb250ZW5kL2Fzc2V0cy9yZWFjdC1yb3V0ZXItZjQ5MzNjZDMuanMiIC8+CiAgICAgICAgPGxpbmsgcmVsPSJtb2R1bGVwcmVsb2FkIiBjcm9zc29yaWdpbiBocmVmPSIvZnJvbnRlbmQvYXNzZXRzL3JlYWN0LXJvdXRlci1kb20tM2Y3MTA5ODAuanMiIC8+CiAgICAgICAgPGxpbmsgcmVsPSJzdHlsZXNoZWV0IiBocmVmPSIvZnJvbnRlbmQvYXNzZXRzL2luZGV4LTVjNmViODZlLmNzcyIgLz4KICAgIDwvaGVhZD4KICAgIDxib2R5PgogICAgICAgIDxkaXYgaWQ9InJvb3QiPjwvZGl2PgogICAgPC9ib2R5Pgo8L2h0bWw+Cg==";
    let html = general_purpose::STANDARD.decode(base64).unwrap();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(html)
}

pub fn frontend_scope() -> Scope {
    let mut scope = Scope::new("frontend");
    scope = scope.route("index.html", web::get().to(fn1));
    scope
}