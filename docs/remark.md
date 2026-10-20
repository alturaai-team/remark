# Remark

Osebni fork Scrawla 0.3.6: zaslon, govor prek Spokenly in lasten izbirnik spletnih elementov. Izvirnik Scrawla: https://github.com/kitlangton/scrawl (MIT). Remark ne potrebuje Agentationa, njegovega strežnika ali sprememb kode ciljne aplikacije.

## Kaj se zajame

- En WAV iz mikrofona, video, kliki in risanje iz Scrawla. Spokenly ga prepiše prek lokalnega MCP `transcribe_file` v JSON. Uporabi model iz **Transcribe File**, ne nujno modela narekovanja.
- Segmentni časi ostanejo merjeni; časov posameznih besed ne izmišljuje. Končni čas segmenta se omeji na dolžino WAV: pri preverjanju je Spokenly za 3816 ms zvoka vrnil 3836 ms.
- Razširitev na že odprti strani izbere element in zajame selector, viewport pravokotnik, pomik, identifikatorje, CSS in razpoložljivo React komponento/izvor. To so podatki strani, brez spreminjanja projektne kode.
- Selector čez odprti shadow root je seznam poti, po ena na root. Koordinate so CSS piksli okvirja strani; niso predstavljene kot piksli posnetka zaslona.
- Vsak okvir ohrani svoj ID in URL; dostopni nadrejeni iframe-i tudi selector, pravokotnik in pomik. Pri okvirjih drugega izvora, katerih nadrejeni DOM ni dostopen, je pot eksplicitno manjkajoča.
- Zajem sprejme največ 300 pripomb; presežek je zavrnjen z istim indikatorjem neuspeha. Nov vklop izbirnika počisti stare vizualne markerje.
- Shranjene pripombe vsebujejo čas, URL, selector in metapodatke. Časovne zastavice dajo oznakam prednost pri izboru slik. Prepis in slike so povezani v povzetku.
- Polja obrazcev in gesla se ne berejo. URL ne vsebuje query parametrov.
- Pri napaki Spokenly je posnetek shranjen; ponudnik se ne zamenja. Neuspešno pošiljanje spletne oznake pokaže `!`.

## Uporaba

1. Zaženi Spokenly in izberi model v Transcribe File.
2. Namesti z `scripts/install-app.sh`: aplikacija je `~/Applications/Remark.app`, CLI `~/bin/remark`, nastavitve `~/.config/remark/config.json`, posnetki `~/Library/Application Support/remark/captures`.
3. Razširitev je v `~/Library/Application Support/remark/browser`. V Chromium brskalniku odpri upravljanje razširitev, vključi razvojni način in naloži to mapo. To je enkratna namestitev na brskalnik.
4. Začni snemanje z bližnjico Remark. V odprtem zavihku klikni ikono razširitve **Označi element**, izberi elemente in govori. Esc izključi izbiranje, zato lahko spet normalno klikaš stran. Klik ikone ga ponovno vključi.
5. Ustavi zajem z bližnjico in po pripravi uporabi Cmd+V pri lokalnem agentu. Povzetek vsebuje absolutne poti; agent odpre slike in manifest na istem Macu.

Razširitev dobi začasen dostop do aktivnega zavihka ob kliku; v kodo strani ni treba dodati knjižnice. Zajete so dovoljene strani in okvirji, do katerih ima razširitev dostop. Zaprti shadow roots in zaščitene sistemske strani niso predstavljeni kot pregledani. Izvorna datoteka/vrstica obstaja samo, kadar jo stran oziroma razvojni build razkrije; enaka omejitev velja tudi za Agentation.

Lokalni povezovalni token je samo v `connection.json` pri razširitvi. Datoteka se ne vključi v Git in token ne pride v JS strani. Razširitev pošilja samo na loopback Remark; zunanji Agentation MCP ni potreben.

Aplikacija se lokalno podpiše kot `dev.altura.remark`, brez posodabljanja izvirnega Scrawla. macOS zato zahteva svoja dovoljenja. Izvirni Scrawl naj med uporabo Remark ne teče, ker imata privzeto enako bližnjico.

## Preverjanje

`taskpolicy -c utility cargo test -j 3 --offline -- --test-threads=2` in `taskpolicy -c utility cargo build -j 3 --offline`. Potrebna sta Rust ter Xcode z Metal Toolchain. Cargo manifest in lockfile ostaneta izvirna.

Sprejem: dve komponenti, govor, dve oznaki, pomik strani in konec zajema. Po enem paste agent prebere pravi prepis ter za vsako oznako čas, URL, selector in povezano sliko. Preveriti je treba tudi geometrijo, shadow root, manjkajoče razvojne metapodatke in ohranitev paketa ob napaki ponudnika.
