# BoringSSL-sys

This library is a low-level wrapper to BoringSSL. Note that since BoringSSL does not offer a stable API, it does not make sense for OS distribution to ship it. Because of that, this library bundles the BoringSSL source code from the `chromium-stable` branch, i.e. the code that Chromium runs.
