# Rust-migratie — werklog

Opdracht: de volledige editor, desktopapp, API en MCP naar Rust; adaptief lint zonder horizontale scrollbar; nieuwe versie publiceren. Werkbranch `rust-migration`, bestaande release 2026.43.1 blijft intact tot de nieuwe app getest is.

Basis: OpenAEC Blueprint `7a2f7cdc095e3854767af07e4e4cf07192498b8f`, gedeelde native iced-bibliotheek 0.4 (`openaec-ui 2026.44.1`), ongewijzigd met bron/licenties in vendor/. Nieuwe appversie 2026.44.0.

Te voltooien en controleren:
- [x] Rust documentmodel: bestaande JSON compatibel, limieten/validatie, geschiedenis en objectselectie.
- [x] Rust renderer: brush/eraser/fill/shapes/tekst/afbeeldingen, alpha, SVG en rasterexports.
- [x] Openen van afbeeldingsformaten, IFCX/IFCXB raster roundtrip, resizing/crop/rotate/flip/correcties.
- [x] Native iced editor: Blueprint-thema's/fonts, tekstvak, vectorobjecten, Ctrl+A/Escape, zoom/pan, conceptopslag.
- [x] Breedtegestuurd lint: uitgebreid → compact → groepsmenu; geen horizontale scrollbar, toetsenbordtoegang.
- [x] Browserpreview uit dezelfde Rust/WASM-app, minimale gegenereerde bindings, geen TS/React-appcode.
- [x] Rust REST API + MCP met auth/loopback/revisies/persistentie en end-to-end tests.
- [ ] Rust builds/installers/associaties/signing, Linux/Windows/macOS CI.
- [x] Oude JS/TS implementatie uit actieve hoofdbron verwijderen nadat functionaliteit overgezet is.
- [ ] Documentatie, screenshots en release 2026.44.0; pas publiceren nadat vereiste controles slagen.

Geen subagents: geen gebruikersopdracht tot delegeren. Private ondertekeningssleutels blijven buiten de repo.


## Actuele voortgang

- Kern: 2 lintbeleidtests + 13 integratietests groen; TIFF/PSD/AVIF/JXL/JP2/HEIC fixtures onafhankelijk met ImageMagick/libheif gegenereerd, geen gebruikersbeelden. Nieuwste HEIC-fixture zit in dezelfde decodeertestlus.
- Services: 3 echte API/MCP/store-integratietests groen. MCP stdio-binary werkt in Rust; procesmatige stdio-test nog toevoegen.
- Native editor: compileert. 5 native headless tests groen, inclusief echte rendering van 5 schermen. PNGs: `crates/paint-app/artifacts/rust-screenshots/` (genegeerd). Smal lint past van 560 tot 1920 px; alle groepsmenu's blijven bereikbaar.
- Browser: dezelfde Rust UI is werkelijk gestart in Chromium op 49325 en rendert goed in WebGL, maar WebGPU bleek kwetsbaar in headless Chromium. Nu wordt de browser opnieuw gebouwd met alleen de gedeelde tiny-skia/softbuffer software-renderer (Cargo feature web is leeg). Die ondersteunt Wasm en vermijdt GPU-afhankelijkheid. Browserdocumenten gaan via Rust/Rexie naar dezelfde IndexedDB `openaec-paint`/`drafts`/`current`, zodat oude drafts kunnen worden hersteld. Nieuwste async-opslagwijzigingen nog e2e controleren.
- Native AVIF: BSD avif-decode/rav1d, lokaal gepatcht voor maximaal 64MP en zonder NASM. Browser AVIF: Rust-adapter rond browser ImageBitmap, daarna PNG naar de gedeelde kern; geen JS-applicatielogica. HEIC, PSD, JXL, JP2 en RAW via Rust-crates. Rawler/DNG end-to-end fixture nog maken.
- `paint-release` Rust CLI implementeert Ed25519 manifestsigning/verificatie (zelfde formaat als 2026.43.1), checksums en Linux-DEB/AppDir. Signingtest en packaging nog draaien. Windows NSIS, macOS Info.plist en Linux-bestandskoppelingen aangemaakt in packaging/.
- Nog niet verwijderen: oude React/TS/Node-bron en workflow pas opruimen nadat nieuwe UI/browser/CLI werkt. Nog te doen: nieuwe Rust CI, docs/release scripts, native releasebuilds, Wasm releasebuild, smoke tests, native install check, 3 nieuwe README screenshots, release publiceren.
- Werklogs in /tmp/paint-*.log. Wasm-bindgen CLI 0.2.130 staat in artifacts/tools/wasm-bindgen-0.2.130-x86_64-unknown-linux-musl/. Debug wasm tot dusver 512MB met debugsecties / 110MB na wasm-bindgen; release moet geoptimaliseerd worden.
- Tijdelijke oude-Rust-preview: python http.server op 49325 (exec session 41942). Nieuwe bestanden in web-dist/. Gebruikerspreview op3020 nog oude versie, bewust niet geraakt; alleen omschakelen wanneer Rust-preview getest is.


## Checkpoint 30 oktober, 22:20
- Werkbranch rust-migration, origin heeft b59f3d3. Oude actieve React/Tauri/Node-code is nu verwijderd (Git behoudt historie). Bron, tests, docs en 3 screenshots worden in volgende checkpoint gepusht.
- Alle 27 Rust tests groen (/tmp/paint-workspace-test.log): 15 core incl. RAW/DNG en afbeelding-opacity, 2 ribbon, 5 UI, 4 service/MCP incl. echte stdio, 1 signing. Rust-verifier verifieert ook eerdere Node-gesigneerde 0.1.0-DEB met bestaande vertrouwde sleutel.
- /tmp/paint-web-final-smoke4.log: volledige browsertest geslaagd, incl echte native browser-bestandskiezer, tekst en draft-reload. RFD 0.16.1 opent bij user activation meteen native filechooser en verbergt HTML-input; tests moeten Playwright filechooser afvangen. Vers-Wasm-doek eerst focus geven en klik met 100ms delay voor betrouwbare test. Extra 7 browsercodec-fixtures nu toegevoegd aan suite, nog lokaal draaien.
- Enorme browservertraging kwam deels van onzichtbaar transparantieraster onder wit canvas. Canvas tekent dit alleen bij transparantie, als één pad. Product-UI toont nu geen Rust-markers overal (alleen info bij instellingen). Laatste eigen screenshots opnieuw gegenereerd.
- Initieel CI 41126151012: alle binaries compileerden (Linux, Windows, beide Mac-architecturen, Wasm); Linux incl DEB/AppImage geslaagd, source-archive geslaagd. Web-testfout was tests die naar HTML-fallback keken, opgelost. Windows packaging NSIS pad niet gevonden: workflow installeert nu expliciet NSIS en ontdekt beide Program Files-paden. Mac packaging lipo -verify_arch argumentvolgorde gecorrigeerd. mac-builds nu parallelle matrix + universal packaging job. cache-on-failure true.
- Nieuwe macOS Rust Cocoa openFiles-bridge toegevoegd; nog op CI compileren/testen. Linux GUI tests blijven goed. Nieuw platformfocusherstel in Rust-web na tekst/open/save, eerstvolgende Wasm-build nodig; cargo check Wasm laatste versie groen.
- Gebruikerspreview omgeschakeld: oude Vite PID102595 gestopt; Python server3020 draait execsession48948 op web-dist. Tijdelijke testserver49325 blijft session41942. CUA IAB tab3 gebonden als paintPreview en herladen; toont eerst laadstatus, nog screenshot van geladen Rust/draft controleren. CUA beleid: alle UI-interacties daar doen; ontwikkeltests zijn scripts in eigen geïsoleerde browsers.
- Lokale release native bin ~36MB, API/MCP ~6MB, tool ~903KB. /tmp/paint-native-final2.log groen, maar laatste focusvelden/code na build moeten nog opnieuw voor definitieve lokale package. CLI native smoke groen. Wasm release23MB na bindings, laatste runtime focustweaks moeten nog rebuild (3min LTO).
- Nog te doen: tweede CI push/run, platformfouten herstellen, alle vereiste controles groen; definitieve Wasm rebuild+browsercodec tests; lokale DEB/installcheck; source archive offline metadata check; signed assets/checksums; fast-forward main, tag v2026.44.0, GitHub release met echte geteste assets publiceren. Nog niets van nieuwe release gepubliceerd. Signingkey blijft buiten repo zoals eerder. Geen subagents.
