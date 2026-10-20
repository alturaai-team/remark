# Remark

Osebni fork Scrawla 0.3.6 za govor, zaslon in strukturirane spletne pripombe. Izvirnik: https://github.com/kitlangton/scrawl (MIT).

## Pogodba

- En posnetek zvoka: Scrawl zajame WAV, lokalni Spokenly MCP ga prepiše z `transcribe_file`, format `json`.
- Uporabi model iz Spokenly **Transcribe File**, ne nujno modela običajnega narekovanja. Besedne čase ne izmišljuje; ohrani časovne segmente.
- Ob napaki ostane posnetek shranjen. Ponudnik se ne zamenja.
- Agentation sejo izbereš izrecno. Paket vsebuje samo pripombe s časom znotraj posnetka. Iz končnega posnetka seje se upoštevajo urejanja in brisanja; že obstoječe pripombe niso vključene.
- Shrani izvorne koordinate, selector in ostale metapodatke Agentation. Dokumentnih koordinat ne razglaša za koordinate zaslona. Pri sliki piše, da je najbližja časovna slika.
- `scrawl.json` ohrani strukturo pripomb, `summary.md` in odložišče pa prepis, čas, URL, selector in metapodatke.
- Izvorna komponenta/datoteka sta znani le, kadar ju zajame spletna aplikacija. Posnetek sam ne zagotavlja vrstice kode.
- Ponovna obdelava ohrani pripombe starega paketa; ne pripne novih pripomb iz trenutno odprte seje.

## Lokalna uporaba

1. Zaženi Spokenly in izberi model v Transcribe File.
2. Zgradi in namesti z `scripts/install-app.sh`. Aplikacija je `~/Applications/Remark.app`; CLI je `~/bin/remark`.
3. Izvedi `remark connect`. Preveri podporo Spokenly in nastavi njegov prepis ter ročno lepljenje. Nastavitve so v `~/.config/remark/config.json`, posnetki v `~/Library/Application Support/remark/captures`.
4. Agentation mora biti naložen na strani in povezan s svojim lokalnim MCP strežnikom. Izvedi `remark agentation sessions`, nato `remark agentation use` z dejanskim ID želene seje. Oba ukaza sprejmeta `--port` (privzeto 4747).
5. Začni/ustavi z bližnjico aplikacije. Govori, klikaj, riši s Shift in dodajaj pripombe Agentation. Po pripravi paketa uporabi Cmd+V pri lokalnem agentu.

Spokenly vrata za povezavo lahko določiš z `remark connect --spokenly-port`. Agentation je poskus izključen do izrecne izbire seje. Izbira seje ne spreminja strani in ne vključuje Agentation v nobeno drugo aplikacijo.

Aplikacija se podpiše lokalno kot `dev.altura.remark`, brez posodabljanja izvirnega Scrawla. macOS zato zahteva svoja dovoljenja za novo aplikacijo. Izvirna aplikacija in njene nastavitve ostanejo ločene. Ker imata privzeto enako bližnjico, naj med uporabo Remark izvirni Scrawl ne teče.

## Preverjanje

Uporabi ukaze iz AGENTS.md: `taskpolicy -c utility cargo test -j 3 --offline -- --test-threads=2` in `taskpolicy -c utility cargo build -j 3 --offline`. Za build sta potrebna Rust in Xcode z Metal Toolchain. Cargo manifest in lockfile ostaneta izvirna.

Sprejemni primer: dve komponenti, govor, dve pripombi, pomik strani, konec zajema. Po enem paste mora agent prebrati pravi prepis in za vsako pripombo čas, selector, URL in sliko; koordinate in izvorna lokacija morajo ohraniti pomen. Izpad Spokenly/Agentation mora biti v paketu viden in ne sme izgubiti posnetka.
