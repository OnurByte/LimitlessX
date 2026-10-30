<div align="center">
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/brand/mark-dark.svg">
  <img src="docs/brand/mark-light.svg" alt="LimitlessX" width="88" height="72">
</picture>

# LimitlessX

**No VPN. No proxy setup. No DPI hacks.**

A browser extension for viewing region-restricted public profiles and posts on X (Twitter).

[![Release](https://img.shields.io/github/v/release/OnurByte/LimitlessX?style=flat&color=555)](https://github.com/OnurByte/LimitlessX/releases/latest)
[![Checks](https://github.com/OnurByte/LimitlessX/actions/workflows/check.yml/badge.svg)](https://github.com/OnurByte/LimitlessX/actions/workflows/check.yml)
[![License](https://img.shields.io/github/license/OnurByte/LimitlessX?style=flat&color=555)](LICENSE)
[![Stars](https://img.shields.io/github/stars/OnurByte/LimitlessX?style=flat&color=555)](https://github.com/OnurByte/LimitlessX/stargazers)

</div>

![LimitlessX off and on: a region-restricted X profile before and after enabling LimitlessX](docs/limitlessx-off-vs-on.png)

## Download

| Chrome, Helium, Brave, Edge, Opera, Vivaldi | Firefox, Zen, LibreWolf, Floorp, Waterfox |
| :---: | :---: |
| <img src="docs/browsers/chrome.svg" alt="Chrome" width="40" height="40"> <img src="docs/browsers/helium.svg" alt="Helium" width="40" height="40"> <img src="docs/browsers/brave.svg" alt="Brave" width="40" height="40"> <img src="docs/browsers/edge.svg" alt="Edge" width="40" height="40"> <img src="docs/browsers/opera.svg" alt="Opera" width="40" height="40"> <img src="docs/browsers/vivaldi.svg" alt="Vivaldi" width="40" height="40"> | <img src="docs/browsers/firefox.svg" alt="Firefox" width="40" height="40"> <img src="docs/browsers/zen-browser.svg" alt="Zen" width="40" height="40"> <img src="docs/browsers/librewolf.svg" alt="LibreWolf" width="40" height="40"> <img src="docs/browsers/floorp.svg" alt="Floorp" width="40" height="40"> <img src="docs/browsers/waterfox-browser.svg" alt="Waterfox" width="40" height="40"> |
| [**Download ZIP**](https://github.com/OnurByte/LimitlessX/releases/latest/download/limitlessx-chromium.zip) | [**Download ZIP**](https://github.com/OnurByte/LimitlessX/releases/latest/download/limitlessx-firefox.zip) |

**Chrome and related browsers:** extract the ZIP, open the Extensions page, enable **Developer mode**, then **Load unpacked** and select the extracted folder.

**Firefox and related browsers:** extract the ZIP, open `about:debugging` → **This Firefox** → **Load Temporary Add-on**, then select `manifest.json`. This unsigned add-on must be loaded again after restarting Firefox.

## Usage

Open a region-restricted profile or post on X. The extension loads the public content automatically. Scroll to load more posts, or enter an `@handle` or X URL in the extension popup.

Content is fetched through [FxTwitter](https://github.com/FxEmbed/FxEmbed), so some profiles or posts may be unavailable. Private accounts are not supported. The extension does not collect X credentials or analytics; viewing counts are stored locally.

[Privacy](docs/privacy.md) · [Browser compatibility](docs/compatibility.md) · [Build from source](docs/build.md) · [Architecture](docs/architecture.md) · [Tests](.github/workflows/check.yml)
