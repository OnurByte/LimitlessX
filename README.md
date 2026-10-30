<div align="center">
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/brand/mark-dark.svg">
  <img src="docs/brand/mark-light.svg" alt="LimitlessX" width="88" height="72">
</picture>

# LimitlessX

**No VPN. No proxy setup. No DPI hacks.**

Read region-restricted public X content directly on X.

[Open source](LICENSE) ⁜ MIT ⁜ Rust + TypeScript

</div>

## Download

| Chrome, Helium, Brave, Edge, Opera, Vivaldi | Firefox |
| :---: | :---: |
| <img src="docs/browsers/chrome.svg" alt="Chrome" width="40" height="40"> <img src="docs/browsers/helium.svg" alt="Helium" width="40" height="40"> <img src="docs/browsers/brave.svg" alt="Brave" width="40" height="40"> <img src="docs/browsers/edge.svg" alt="Edge" width="40" height="40"> <img src="docs/browsers/opera.svg" alt="Opera" width="40" height="40"> <img src="docs/browsers/vivaldi.svg" alt="Vivaldi" width="40" height="40"> | <img src="docs/browsers/firefox.svg" alt="Firefox" width="48" height="48"> |
| [**Download ZIP**](https://github.com/OnurByte/LimitlessX/releases/latest/download/limitlessx-chromium.zip) | [**Download ZIP**](https://github.com/OnurByte/LimitlessX/releases/latest/download/limitlessx-firefox.zip) |

**Chrome and related browsers:** extract the ZIP, open the Extensions page, enable **Developer mode**, then **Load unpacked** and select the extracted folder.

**Firefox:** extract the ZIP, open `about:debugging` → **This Firefox** → **Load Temporary Add-on**, then select `manifest.json`. This unsigned add-on must be loaded again after restarting Firefox.

## Use

Open a region-restricted profile or post on X. Available public content appears automatically. Five posts load first; scroll for more. The popup accepts an `@handle` or X URL and keeps your viewing counts locally.

Public data comes from [FxTwitter](https://github.com/FxEmbed/FxEmbed). Availability varies; private accounts are unavailable. No X credentials or analytics. [Privacy](docs/privacy.md) ⁜ [Browser compatibility](docs/compatibility.md)

[Build from source](docs/build.md) ⁜ [Architecture](docs/architecture.md) ⁜ [Tests](.github/workflows/check.yml)
