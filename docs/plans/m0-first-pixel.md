# Erk Engine M0 (İlk Piksel) Uygulama Planı

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `erk examples/merhaba.html` yerel bir HTML dosyasını açıp Türkçe
başlık ve paragrafı biçimli olarak bir pencerede gösteriyor; `erk --screenshot
out.png examples/merhaba.html` aynı sayfayı başsız modda deterministik bir PNG'ye
yazıyor ve bu PNG bir altın dosya testiyle sabitleniyor.

**Architecture:** Tek süreç, ağ yok. `erk-dom` arena DOM'dur ve html5ever'ın
`TreeSink`'ini uygular. `erk-renderer` Stylo ile stil, Taffy ile block layout,
Parley ile paragraf şekillendirme yapar, bir display list üretir ve `vello_cpu`
ile boyar. `erk-shell` winit + softbuffer penceresini çalıştırır; renderer kendi
iş parçacığındadır ve kabukla yalnızca tipli mesajlarla (`std::sync::mpsc`)
konuşur. Gerekçeler: [p0-architecture.md](../design/p0-architecture.md).

**Tech Stack:** Rust stable, html5ever 0.39.0, stylo (Blitz adaptörüyle aynı
sürüm, bkz. Global Constraints), taffy 0.14, parley 0.11, vello_cpu 0.2,
winit 0.30, softbuffer 0.4, png.

## Global Constraints

Bu kısıtlar her görevin gereksinimlerine örtük olarak dahildir.

- **Toolchain:** Sürüm `rust-toolchain.toml`'da sabit ve `rustc --version`
  çıktısından alınır. Bağımlılıkların MSRV tabanı ≥ 1.89 (blitz-dom 1.89,
  parley 1.88, wgpu 1.87); kurulu stable bunun altındaysa yükseltilir.
- **html5ever tam sürüm sabit:** `html5ever = "=0.39.0"`. 0.40 `web_atoms` 0.3'e
  geçti ve Stylo ile atom tipleri uyuşmuyor. `cargo update` html5ever'ı
  kaldırmamalı.
- **Stylo sürümü:** Uyarlanan adaptörün derlendiği sürüm. Kaynak blitz-dom
  0.3.0-beta.2 ve o `stylo ^0.20.0` kullanıyor, dolayısıyla M0'da Stylo 0.20.x.
  `stylo_traits`, `stylo_dom`, `selectors`, `stylo_taffy` sürümleri blitz-dom
  0.3.0-beta.2'nin `Cargo.toml`'undan alınır. 0.21'e yükseltme M0 bittikten
  sonra ayrı bir PR'dır.
- **Python 3:** Stylo'nun `build.rs`'i `properties/build.py`'yi çalıştırır.
  Önce `PYTHON3` ortam değişkenine, yoksa Windows'ta `python.exe`'ye bakar.
  LLVM gerekmez (bindgen yalnızca `gecko` özelliğinde).
- **`unsafe` yasak:** workspace lint'i `forbid`. Bu taşta istisna yok.
- **`erk-dom` yapraktır:** başka `erk-*` crate'e bağımlı olmaz. Stylo'yu da
  bilmez; stil verisi `erk-renderer`'daki yan tabloda durur (Task 3).
- **Kabuk DOM'a dokunamaz:** `erk-shell`'in doğrudan bağımlılıkları arasında
  `erk-dom` yok. `erk-renderer`, `erk-dom` tiplerini dışa açmaz.
- **Mesajlar sahipli veri taşır:** kabuk ↔ renderer mesajlarında referans,
  `Arc`, `Mutex` yok. M3'te serileştirilebilir olmaları bir türetme satırı
  olmalı.
- **Belirleyicilik:** Altın testler depodaki gömülü yazı tipini kullanır, sistem
  yazı tipini değil. Sabit boyut (800×600), 1x DPI, `vello_cpu` tek iş
  parçacığında.
- **Referans kodun kullanımı:** Blitz (MIT OR Apache-2.0) kodu uyarlanırken
  dosya başına kaynak yorumu yazılır (`Adapted from blitz-dom 0.3.0-beta.2,
  src/stylo.rs`). MPL-2.0 bir dosya (`stylo_taffy`, Servo) **kopyalanmaz**,
  bağımlılık olarak kullanılır.
- **Test disiplini:** Her görev testle başlar. `cargo test --workspace` yeşil
  olmadan commit yapılmaz; derlenmemiş kod commit'lenmez.
- **Kod dili:** Tanımlayıcılar, yorumlar ve commit mesajları İngilizce.
- **API doğrulaması:** Aşağıdaki API adları araştırma anındaki sürümlerden
  alındı. Her görevin ilk adımı ilgili crate'in pinlenen sürümünün docs.rs
  sayfasını ya da kaynağını okumaktır. Plan ile gerçek farklıysa gerçek
  kazanır ve fark Yürütme Notları'na yazılır.

## Dosya Yapısı

```
rust-toolchain.toml                     Sabit toolchain
Cargo.toml                              workspace, [workspace.lints]
.github/workflows/ci.yml                Windows + Ubuntu, muhafız adımları
examples/merhaba.html                   Kabul sayfası

crates/erk-dom/clippy.toml              disallowed-types: Rc
crates/erk-dom/src/lib.rs               Genel API
crates/erk-dom/src/arena.rs             Arena<T>, NodeId
crates/erk-dom/src/node.rs              Node, NodeData, ElementData
crates/erk-dom/src/document.rs          Document: ağaç işlemleri, gezinme
crates/erk-dom/src/sink.rs              html5ever TreeSink
crates/erk-dom/tests/parse.rs           Ayrıştırma testleri

crates/erk-renderer/src/lib.rs          spawn(), mesaj tipleri (tek genel yüzey)
crates/erk-renderer/src/messages.rs     ToRenderer / FromRenderer
crates/erk-renderer/src/style/mod.rs    Stylist, Device, traverse
crates/erk-renderer/src/style/node.rs   TNode/TElement adaptörü (Blitz'ten uyarlanır)
crates/erk-renderer/src/style/side.rs   NodeId.index ile Stylo yan tablosu
crates/erk-renderer/src/style/ua.css    Kullanıcı ajanı stil sayfası
crates/erk-renderer/src/layout/mod.rs   Taffy low-level trait'leri
crates/erk-renderer/src/layout/text.rs  Parley paragraf yaprağı
crates/erk-renderer/src/display.rs      DisplayItem, display list kurma
crates/erk-renderer/src/paint.rs        vello_cpu boyama
crates/erk-renderer/assets/fonts/       Gömülü, OFL lisanslı yazı tipi
crates/erk-renderer/tests/golden.rs     Altın PNG testi
crates/erk-renderer/tests/golden/       Referans PNG'ler

crates/erk-shell/src/main.rs            CLI: pencere modu ve --screenshot
crates/erk-shell/src/window.rs          winit ApplicationHandler, softbuffer
```

---

### Task 1: Muhafızlar ve toolchain

Bu taşın muhafızları bu görevde ve korudukları şeyle gelen görevlerde (Task 2,
3, 7) kurulur.

**Files:**
- Create: `rust-toolchain.toml`
- Modify: `Cargo.toml`, `crates/*/Cargo.toml`, `.github/workflows/ci.yml`

- [ ] **Step 1: Ortamı doğrula**

```bash
rustc --version
cargo --version
python --version
```

Beklenen: üçü de sürüm basar. Sürümler Yürütme Notları'na yazılır.

- [ ] **Step 2: Toolchain'i sabitle**

```toml
# rust-toolchain.toml
[toolchain]
channel = "<rustc --version çıktısındaki sürüm, ör. 1.98.0>"
components = ["rustfmt", "clippy"]
```

Kök `Cargo.toml`'da `[workspace.package]` altına aynı sürümle
`rust-version = "..."` eklenir.

- [ ] **Step 3: Workspace lint'leri**

Kök `Cargo.toml`:

```toml
[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
```

Her `crates/*/Cargo.toml`:

```toml
[lints]
workspace = true
```

- [ ] **Step 4: Kasıtlı ihlal: `unsafe`**

`crates/erk-dom/src/lib.rs` içine geçici olarak:

```rust
pub fn violation() { unsafe {} }
```

Çalıştır: `cargo build -p erk-dom`. Beklenen: `unsafe_code` forbid hatasıyla
başarısız. Satırı sil, tekrar derle. Beklenen: başarılı.

- [ ] **Step 5: CI — Windows ve lint devralma kontrolü**

`.github/workflows/ci.yml` içinde job'a `strategy.matrix.os: [ubuntu-latest,
windows-latest]` eklenir. Toolchain adımı `rust-toolchain.toml`'u okur
(`rustup toolchain install` argümansız çalıştığında dosyadaki toolchain'i
kurar; bunun kurulu rustup sürümünde böyle olduğu doğrulanır). Yeni adım:

```yaml
- name: Every crate inherits the workspace lints
  shell: bash
  run: |
    fail=0
    for f in crates/*/Cargo.toml; do
      if ! grep -Pzq '\[lints\]\s*\nworkspace = true' "$f"; then
        echo "$f does not inherit [workspace.lints]"; fail=1
      fi
    done
    exit $fail
```

Kasıtlı ihlal: bir crate'ten `[lints]` bloğunu sil, aynı betiği yerelde (Git
Bash) çalıştır. Beklenen: hata. Geri al.

- [ ] **Step 6: Doğrula ve commit'le**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Beklenen: üçü de yeşil. Commit: `chore: pin toolchain and forbid unsafe
workspace-wide`.

---

### Task 2: `erk-dom` — arena, düğümler, ayrıştırma

**Files:**
- Create: `crates/erk-dom/clippy.toml`, `src/arena.rs`, `src/node.rs`,
  `src/document.rs`, `src/sink.rs`, `tests/parse.rs`
- Modify: `crates/erk-dom/Cargo.toml` (`html5ever = "=0.39.0"`), `src/lib.rs`

**Interfaces:**
- Produces: `NodeId`, `Arena<T>`, `Document`, `Node`, `NodeData`,
  `ElementData`, `Document::parse_html(&str) -> Document`, gezinme
  (`children(id)`, `parent(id)`, `node(id)`)

- [ ] **Step 1: `Rc` muhafızı**

```toml
# crates/erk-dom/clippy.toml
disallowed-types = [
  { path = "std::rc::Rc", reason = "DOM nodes are owned by the arena and addressed by NodeId (docs/design/p0-architecture.md §5)" },
  { path = "std::rc::Weak", reason = "see std::rc::Rc" },
]
```

Kasıtlı ihlal: `lib.rs`'e `pub struct Violation(std::rc::Rc<u8>);`, sonra
`cargo clippy -p erk-dom -- -D warnings`. Beklenen: `disallowed_types` hatası.
Geri al.

- [ ] **Step 2: Arena testlerini yaz (önce kırmızı)**

`src/arena.rs` içindeki `#[cfg(test)] mod tests`:

- ekle → al: değer geri geliyor
- sil → eski `NodeId` ile al: `None`
- sil → yeniden ekle: aynı indeks, **farklı** nesil; eski `NodeId` hâlâ `None`
- nesli tükenen slot (test için nesli `u32::MAX`'a ayarlayan `#[cfg(test)]`
  yardımcı) silindiğinde serbest listeye **girmez**
- `size_of::<Option<NodeId>>() == 8`

- [ ] **Step 3: Arenayı yaz**

```rust
use std::num::NonZeroU32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId {
    index: u32,
    generation: NonZeroU32,
}

impl NodeId {
    pub fn index(self) -> u32 {
        self.index
    }
}

struct Slot<T> {
    generation: NonZeroU32,
    value: Option<T>,
}

pub struct Arena<T> {
    slots: Vec<Slot<T>>,
    free: Vec<u32>,
}

impl<T> Arena<T> {
    pub fn insert(&mut self, value: T) -> NodeId {
        if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index as usize];
            slot.value = Some(value);
            return NodeId { index, generation: slot.generation };
        }
        let index = u32::try_from(self.slots.len()).expect("arena holds at most u32::MAX nodes");
        let generation = NonZeroU32::MIN;
        self.slots.push(Slot { generation, value: Some(value) });
        NodeId { index, generation }
    }

    pub fn remove(&mut self, id: NodeId) -> Option<T> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation != id.generation {
            return None;
        }
        let value = slot.value.take()?;
        // A wrapped generation would make a stale NodeId valid again, so an
        // exhausted slot is retired instead of reused.
        if let Some(next) = slot.generation.checked_add(1) {
            slot.generation = next;
            self.free.push(id.index);
        }
        Some(value)
    }

    pub fn get(&self, id: NodeId) -> Option<&T> {
        let slot = self.slots.get(id.index as usize)?;
        if slot.generation != id.generation {
            return None;
        }
        slot.value.as_ref()
    }

    // get_mut: same checks, &mut
}
```

Çalıştır: `cargo test -p erk-dom`. Beklenen: arena testleri yeşil.

- [ ] **Step 4: Düğüm modeli ve ağaç işlemleri**

`Node`: `parent`, `first_child`, `last_child`, `prev_sibling`, `next_sibling`
(hepsi `Option<NodeId>`) ve `data: NodeData`. `NodeData`: `Document`,
`Doctype { name, public_id, system_id }`, `Element(ElementData)`, `Text`,
`Comment`, `ProcessingInstruction`. `ElementData`: `QualName`, öznitelikler,
`template_contents: Option<NodeId>`, `mathml_annotation_xml_integration_point`.

`Document` arenayı tutar ve şu işlemleri sağlar: `append(parent, child)`,
`insert_before(sibling, child)`, `detach(id)`, `children(id)` yineleyicisi.
Testler: ekleme sırası, `insert_before` başa ekleme, `detach` sonrası kardeş
bağlarının onarılması.

- [ ] **Step 5: `TreeSink`**

html5ever 0.39'un `TreeSink`'i `&self` alır; `Document` bu yüzden bir `RefCell`
içinde durur (`RefCell` yasak değil, `Rc` yasak). `Handle = NodeId`.

Önce docs.rs/html5ever/0.39.0 üzerinde `TreeSink`'in zorunlu metotları ve
`ElemName` ilişkili tipi okunur. Referans uygulama: blitz-html 0.3.0-beta.2'nin
sink'i (aynı html5ever sürümü). Dikkat edilecekler:

- ardışık metin eklemeleri **tek bir metin düğümünde birleşir** (tokenizer bir
  paragrafı birkaç parça halinde verebilir)
- `append_based_on_parent_node`, `append_before_sibling`, `reparent_children`
  ve `add_attrs_if_missing` tablo ve biçimlendirme öğesi düzeltmeleri için
  gerekli, stub bırakılmaz
- `get_template_contents` `<template>` için ayrı bir belge parçası düğümü döner

- [ ] **Step 6: Ayrıştırma testleri**

`tests/parse.rs`:

- `<p>Merhaba <b>dünya</b></p>` → `html > head, body > p > ["Merhaba ", b > ["dünya"]]`;
  `html`, `head`, `body` örtük olarak oluşmuş
- `<p>a&amp;b</p>` → `p`'nin tek bir metin çocuğu var: `"a&b"`
- `İıŞşĞğÜüÖöÇç` bayt bayt korunuyor
- `<table><tr><td>x</td></tr></table>` → örtük `tbody` var
- `<b><p>x</b>y</p>` → biçimlendirme öğesi yeniden inşası (adoption agency)
  html5lib'in beklediği ağacı veriyor

- [ ] **Step 7: Yaprak muhafızı**

CI'ya:

```yaml
- name: erk-dom depends on no other workspace crate
  shell: bash
  run: |
    if cargo tree -p erk-dom -e normal --prefix none | grep -E '^erk-' | grep -v '^erk-dom '; then
      echo "erk-dom must stay a leaf crate"; exit 1
    fi
```

Kasıtlı ihlal: `erk-dom`'a `erk-network = { path = "../erk-network" }`, betik
yerelde. Beklenen: hata. Geri al.

- [ ] **Step 8: Doğrula ve commit'le**

fmt, clippy, test yeşil. Commit: `feat(dom): arena DOM with generational node
ids and html5ever sink`.

---

### Task 3: Stil — Stylo

**Files:**
- Create: `crates/erk-renderer/src/style/{mod.rs,node.rs,side.rs,ua.css}`
- Modify: `crates/erk-renderer/Cargo.toml`

**Interfaces:**
- Consumes: `erk_dom::{Document, NodeId}`
- Produces: `StyleEngine::new(viewport) `, `restyle(&Document)`,
  `computed(NodeId) -> Option<Arc<ComputedValues>>` (crate içi)

- [ ] **Step 1: Stylo'yu tek başına derle**

blitz-dom 0.3.0-beta.2'nin `Cargo.toml`'undan `stylo`, `stylo_traits`,
`stylo_dom`, `selectors`, `stylo_atoms`, `stylo_taffy` ve `atomic_refcell`
sürümleri alınıp `erk-renderer`'a eklenir. `cargo build -p erk-renderer`.

Beklenen: derleniyor. Python'un bulunduğu, süre ve `target/` boyutu Yürütme
Notları'na yazılır. Python bulunamazsa `PYTHON3` ayarlanıp tekrar denenir.

- [ ] **Step 2: Atom muhafızı**

```yaml
- name: html5ever and Stylo share one atom crate version
  shell: bash
  run: |
    if cargo tree -d -e normal | grep -E '^(web_atoms|string_cache) '; then
      echo "duplicate atom crates: upgrade html5ever and stylo together"; exit 1
    fi
```

Kasıtlı ihlal: `erk-dom`'da `html5ever = "=0.40.1"`. Beklenen: ya derleme ya bu
adım kırılır; hangisinin kırıldığı Yürütme Notları'na yazılır. Geri al.

- [ ] **Step 3: Adaptörü uyarla**

Blitz'in `blitz-dom/src/stylo.rs`'i (`TDocument`, `TNode`, `TShadowRoot`,
`NodeInfo`, `TElement`, `selectors::Element`, `DomTraversal`) birebir
uyarlanır. Tutamak: `ErkNode<'a> { doc: &'a Document, side: &'a StyleSide, id:
NodeId }` (`Copy`, `Eq`/`Hash` `id` üzerinden). Özgünlük aranmaz.

**Blitz'ten tek bilinçli sapma:** Stylo'nun düğüm başına verisi
(`AtomicRefCell<Option<ElementData>>`, kirli bayrakları) DOM düğümünde değil,
`StyleSide` yan tablosunda `NodeId::index()` ile durur. Sebebi `erk-dom`'un
yaprak kalması ve stil motorunu bilmemesi. Bir trait imzası bunu imkânsız
kılarsa (ör. düğüme bağlı bir ömür ya da `'static` isterse), Blitz modeline
dönülür ve sebep Yürütme Notları'na yazılır.

`TShadowRoot` stub. UA stil sayfası Blitz'inkinden alınır (lisansı dosya
başında belirtilir).

- [ ] **Step 4: Hesaplanmış stil testleri**

- `<style>p { color: red }</style><p>x</p>` → `p`'nin rengi kırmızı
- `<h1>` UA stil sayfasından `display: block` ve varsayılan boyutundan büyük
  `font-size` alıyor
- `<p style="margin-top: 7px">` → hesaplanmış `margin-top` 7px
- kalıtım: `body { color: blue }` → `p` içindeki metin mavi

- [ ] **Step 5: Doğrula ve commit'le**

Commit: `feat(style): style the arena DOM with Stylo`. Gövde, adaptörün
Blitz'ten uyarlandığını ve yan tablo sapmasının sebebini söyler.

---

### Task 4: Layout — Taffy block

**Files:**
- Create: `crates/erk-renderer/src/layout/mod.rs`

- [ ] **Step 1:** Taffy 0.14'ün low-level trait'leri (`TraversePartialTree`,
  `LayoutPartialTree`, `CacheTree` ve ilgili) docs.rs'ten okunur. Layout ağacı
  Erk'in DOM'udur; düğüm başına `taffy::Style`, `Cache` ve `Layout` bir yan
  tabloda (`NodeId::index()`) durur.
- [ ] **Step 2:** Stylo'nun `ComputedValues`'u `stylo_taffy` ile
  `taffy::Style`'a çevrilir (bağımlılık olarak; MPL dosyası kopyalanmaz).
  `display: none` alt ağaçları layout'a girmez.
- [ ] **Step 3: Testler:**
  - `width: 100px` bir `div` → layout genişliği 100
  - iki blok kardeş alt alta; toplam yükseklik ikisinin toplamı
  - `margin-top` konumu kaydırıyor
  - iki kardeş arasında margin collapsing (Taffy'nin block layout'u bunu
    yapıyor mu burada ölçülür; yapmıyorsa bu M1'in akış layout'u kararına veri
    olur ve Yürütme Notları'na yazılır)
- [ ] **Step 4:** Commit: `feat(layout): block layout on the DOM with Taffy`.

---

### Task 5: Paragraf yaprağı — Parley

**Tam IFC değil.** Satırlar arası span kırılması, satır içi görseller ve karışık
stiller M1'de.

**Files:**
- Create: `crates/erk-renderer/src/layout/text.rs`, `assets/fonts/`

- [ ] **Step 1: Gömülü yazı tipi.** Türkçe glifleri içeren OFL lisanslı bir yazı
  tipi (ör. Noto Sans Regular) `assets/fonts/` altına, lisans dosyasıyla birlikte
  eklenir. Dosyanın indirilmesi için kullanıcıdan onay alınır. Altın testler
  yalnızca bu yazı tipini kullanır.
- [ ] **Step 2:** Çocukları yalnızca metin olan blok (M0'da inline öğeler
  metinleri ebeveynin stiliyle birleştirilir), Taffy'de **ölçüm fonksiyonlu bir
  yapraktır**. Ölçüm fonksiyonu Parley ile metni ebeveynin hesaplanmış yazı
  tipi, boyutu ve satır yüksekliğiyle şekillendirir, verilen genişlikte satırlara
  böler ve `(genişlik, yükseklik)` döner.
- [ ] **Step 3:** Parley layout'u `NodeId` ile önbelleğe alınır; boyama aynı
  layout'u kullanır, yeniden şekillendirmez.
- [ ] **Step 4: Testler:**
  - uzun paragraf dar genişlikte birden fazla satıra bölünüyor, yüksekliği satır
    sayısıyla orantılı
  - `İstanbul` şekillendiriliyor ve her karakter için bir glif var (gömülü yazı
    tipinde eksik glif yok)
- [ ] **Step 5:** Commit: `feat(layout): shape paragraphs with Parley as Taffy
  leaves`.

---

### Task 6: Display list ve boyama

**Files:**
- Create: `crates/erk-renderer/src/display.rs`, `src/paint.rs`,
  `tests/golden.rs`, `tests/golden/merhaba.png`

- [ ] **Step 1:** `DisplayItem`: `Rect { rect, color }` (arka plan) ve
  `GlyphRun { font, size, color, glyphs }`. Layout ağacı gezilerek kurulur:
  önce arka planlar, sonra metin. Display list metin olarak da dökülebilir
  (hata ayıklama ve ileride reftest için).
- [ ] **Step 2:** `vello_cpu` 0.2'nin API'si docs.rs'ten okunur (render bağlamı,
  paint ayarı, dikdörtgen doldurma, glif çizme, pixmap'e boyama). Display list
  sırayla boyanır; tek iş parçacığı.
- [ ] **Step 3:** PNG'ye yazma (`png` crate'i).
- [ ] **Step 4: Altın test.** `tests/golden.rs` `examples/merhaba.html`'i
  800×600'de boyar ve `tests/golden/merhaba.png` ile piksel piksel karşılaştırır.
  Uyuşmazlıkta gerçek çıktı `target/golden-actual/` altına yazılır ki fark
  incelenebilsin. İlk referans elle incelenip commit'lenir.
- [ ] **Step 5: Kasıtlı ihlal:** Varsayılan metin rengini değiştir → test kırmızı.
  Geri al.
- [ ] **Step 6:** Commit: `feat(paint): display list painted with vello_cpu`.

---

### Task 7: Kabuk — pencere ve renderer iş parçacığı

**Files:**
- Create: `crates/erk-renderer/src/messages.rs`, `crates/erk-shell/src/window.rs`
- Modify: `crates/erk-renderer/src/lib.rs`, `crates/erk-shell/src/main.rs`,
  `crates/erk-shell/Cargo.toml`

**Interfaces:**
- `erk_renderer::spawn() -> (Sender<ToRenderer>, Receiver<FromRenderer>)`
- `ToRenderer::{Load { path: PathBuf }, Resize { width, height, scale }, Shutdown}`
- `FromRenderer::{Frame { width, height, pixels: Vec<u32> }, Error { message: String }}`

- [ ] **Step 1:** Renderer kendi iş parçacığında döngüde mesaj bekler. Genel yüzey
  yalnızca `spawn` ve mesaj tipleri; `erk-dom` tipleri dışa açılmaz.
- [ ] **Step 2:** `erk-shell`: winit 0.30 `ApplicationHandler` + softbuffer 0.4.
  Renderer'dan gelen kareler için kabukta ayrı bir iş parçacığı `Receiver`'da
  bekler ve winit'i `EventLoopProxy` ile uyandırır. Renderer winit'i bilmez.
- [ ] **Step 3:** CLI: `erk <dosya>` pencere; `erk --screenshot <çıktı.png>
  <dosya>` pencere açmadan aynı yoldan PNG yazar.
- [ ] **Step 4: Kabuk muhafızı**

```yaml
- name: erk-shell does not depend on erk-dom
  shell: bash
  run: |
    if cargo tree -p erk-shell -e normal --depth 1 --prefix none | grep -E '^erk-dom '; then
      echo "the shell talks to the renderer through messages, not the DOM"; exit 1
    fi
```

`--depth 1` bilerek: `erk-shell → erk-renderer → erk-dom` zinciri dolaylı
olarak her zaman görünür; yasak olan doğrudan bağımlılık. Kasıtlı ihlal:
`erk-shell`'e `erk-dom` ekle → hata. Geri al.

- [ ] **Step 5: Test:** Renderer'a `Load` + `Resize` gönder, bir `Frame` gelsin;
  boyutu istenenle aynı. `Shutdown` sonrası iş parçacığı sonlanıyor.
- [ ] **Step 6:** Commit: `feat(shell): window driven by a renderer thread over
  messages`.

---

### Task 8: Kabul

- [ ] `examples/merhaba.html`: Türkçe bir `<h1>` ve iki paragraf, bir arka plan
  rengi, bir `<style>` bloğu.
- [ ] `cargo run -p erk-shell -- examples/merhaba.html` pencerede biçimli metni
  gösteriyor (elle doğrulanır; ekran görüntüsü PR'a eklenir).
- [ ] `cargo run -p erk-shell -- --screenshot out.png examples/merhaba.html`
  altın PNG ile aynı.
- [ ] README'ye çalıştırma komutları; `roadmap.md`'de M0 durumu "Bitti".
- [ ] [p0-verification.md](../design/p0-verification.md) tablosunda M0
  satırlarının "Denendi" sütunu dolduruldu.

---

## Plan Öz-Denetimi

Planı yazdıktan sonra tasarım dokümanına ve yol haritasına karşı kontrol ettim.

| Tasarım / yol haritası | Karşılayan görev |
|---|---|
| Tasarım §2.2 mesaj disiplini | Task 7 (mesaj tipleri, kabuk muhafızı) |
| Tasarım §5.1 arena ve `NodeId` | Task 2 Step 2–3 |
| Tasarım §5.2 iç değiştirilebilirlik | Task 2 Step 5 |
| Tasarım §6.1 Stylo, Blitz adaptörü | Task 3 |
| Tasarım §6.2 Taffy low-level, `stylo_taffy` | Task 4 |
| Tasarım §6.2 M0 paragraf yaprağı | Task 5 |
| Tasarım §6.3 display list, `vello_cpu` | Task 6 |
| Doğrulama §1 M0 muhafızları | Task 1 (unsafe, lint devralma), 2 (Rc, yaprak), 3 (atom), 6 (altın PNG), 7 (kabuk) |
| Doğrulama §3.1 belirleyicilik | Global Constraints, Task 5 Step 1, Task 6 Step 4 |
| Doğrulama §4 CI | Task 1 Step 5 |
| Yol haritası M0 kabulü | Task 8 |

**Bilinen belirsizlikler:** Stylo adaptörünün yan tablo sapması (Task 3 Step 3),
Taffy'nin margin collapsing davranışı (Task 4 Step 3) ve rustup'ın argümansız
`toolchain install` davranışı (Task 1 Step 5) yürütmede doğrulanacak.

---

## Yürütme Notları

*(Görevler yürütüldükçe, planın yanlış çıkan varsayımları ve doğrulanan gerçeklerle
doldurulur.)*
