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
- **`unsafe` yasak:** workspace lint'i `forbid`. Tek istisna `erk-style`: Stylo'nun
  `TElement`'i beş metodu `unsafe fn` olarak tanımlıyor (bkz. Task 3 yürütme notları).
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
  src/stylo.rs`). MPL-2.0 bir dosya (ör. Servo veya Firefox kaynağı) **kopyalanmaz**,
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

- [x] **Step 1: Ortamı doğrula**

```bash
rustc --version
cargo --version
python --version
```

Beklenen: üçü de sürüm basar. Sürümler Yürütme Notları'na yazılır.

- [x] **Step 2: Toolchain'i sabitle**

```toml
# rust-toolchain.toml
[toolchain]
channel = "<rustc --version çıktısındaki sürüm, ör. 1.98.0>"
components = ["rustfmt", "clippy"]
```

Kök `Cargo.toml`'da `[workspace.package]` altına aynı sürümle
`rust-version = "..."` eklenir.

- [x] **Step 3: Workspace lint'leri**

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

- [x] **Step 4: Kasıtlı ihlal: `unsafe`**

`crates/erk-dom/src/lib.rs` içine geçici olarak:

```rust
pub fn violation() { unsafe {} }
```

Çalıştır: `cargo build -p erk-dom`. Beklenen: `unsafe_code` forbid hatasıyla
başarısız. Satırı sil, tekrar derle. Beklenen: başarılı.

- [x] **Step 5: CI — Windows ve lint devralma kontrolü**

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

- [x] **Step 6: Doğrula ve commit'le**

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

- [x] **Step 1: `Rc` muhafızı**

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

- [x] **Step 2: Arena testlerini yaz (önce kırmızı)**

`src/arena.rs` içindeki `#[cfg(test)] mod tests`:

- ekle → al: değer geri geliyor
- sil → eski `NodeId` ile al: `None`
- sil → yeniden ekle: aynı indeks, **farklı** nesil; eski `NodeId` hâlâ `None`
- nesli tükenen slot (test için nesli `u32::MAX`'a ayarlayan `#[cfg(test)]`
  yardımcı) silindiğinde serbest listeye **girmez**
- `size_of::<Option<NodeId>>() == 8`

- [x] **Step 3: Arenayı yaz**

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

- [x] **Step 4: Düğüm modeli ve ağaç işlemleri**

`Node`: `parent`, `first_child`, `last_child`, `prev_sibling`, `next_sibling`
(hepsi `Option<NodeId>`) ve `data: NodeData`. `NodeData`: `Document`,
`Doctype { name, public_id, system_id }`, `Element(ElementData)`, `Text`,
`Comment`, `ProcessingInstruction`. `ElementData`: `QualName`, öznitelikler,
`template_contents: Option<NodeId>`, `mathml_annotation_xml_integration_point`.

`Document` arenayı tutar ve şu işlemleri sağlar: `append(parent, child)`,
`insert_before(sibling, child)`, `detach(id)`, `children(id)` yineleyicisi.
Testler: ekleme sırası, `insert_before` başa ekleme, `detach` sonrası kardeş
bağlarının onarılması.

- [x] **Step 5: `TreeSink`**

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

- [x] **Step 6: Ayrıştırma testleri**

`tests/parse.rs`:

- `<p>Merhaba <b>dünya</b></p>` → `html > head, body > p > ["Merhaba ", b > ["dünya"]]`;
  `html`, `head`, `body` örtük olarak oluşmuş
- `<p>a&amp;b</p>` → `p`'nin tek bir metin çocuğu var: `"a&b"`
- `İıŞşĞğÜüÖöÇç` bayt bayt korunuyor
- `<table><tr><td>x</td></tr></table>` → örtük `tbody` var
- `<b><p>x</b>y</p>` → biçimlendirme öğesi yeniden inşası (adoption agency)
  html5lib'in beklediği ağacı veriyor

- [x] **Step 7: Yaprak muhafızı**

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

- [x] **Step 8: Doğrula ve commit'le**

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

- [x] **Step 1: Stylo'yu tek başına derle**

blitz-dom 0.3.0-beta.2'nin `Cargo.toml`'undan `stylo`, `stylo_traits`,
`stylo_dom`, `selectors`, `stylo_atoms`, `stylo_taffy` ve `atomic_refcell`
sürümleri alınıp `erk-renderer`'a eklenir. `cargo build -p erk-renderer`.

Beklenen: derleniyor. Python'un bulunduğu, süre ve `target/` boyutu Yürütme
Notları'na yazılır. Python bulunamazsa `PYTHON3` ayarlanıp tekrar denenir.

- [x] **Step 2: Atom muhafızı**

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

- [x] **Step 3: Adaptörü uyarla**

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

- [x] **Step 4: Hesaplanmış stil testleri**

- `<style>p { color: red }</style><p>x</p>` → `p`'nin rengi kırmızı
- `<h1>` UA stil sayfasından `display: block` ve varsayılan boyutundan büyük
  `font-size` alıyor
- `<p style="margin-top: 7px">` → hesaplanmış `margin-top` 7px
- kalıtım: `body { color: blue }` → `p` içindeki metin mavi

- [x] **Step 5: Doğrula ve commit'le**

Commit: `feat(style): style the arena DOM with Stylo`. Gövde, adaptörün
Blitz'ten uyarlandığını ve yan tablo sapmasının sebebini söyler.

---

### Task 4: Layout — Taffy block

**Files:**
- Create: `crates/erk-renderer/src/layout/mod.rs`

- [x] **Step 1:** Taffy 0.14'ün low-level trait'leri (`TraversePartialTree`,
  `LayoutPartialTree`, `CacheTree` ve ilgili) docs.rs'ten okunur. Layout ağacı
  Erk'in DOM'udur; düğüm başına `taffy::Style`, `Cache` ve `Layout` bir yan
  tabloda (`NodeId::index()`) durur.
- [x] **Step 2:** Stylo'nun `ComputedValues`'u `stylo_taffy` ile
  `taffy::Style`'a çevrilir (bağımlılık olarak; MPL dosyası kopyalanmaz).
  `display: none` alt ağaçları layout'a girmez.
- [x] **Step 3: Testler:**
  - `width: 100px` bir `div` → layout genişliği 100
  - iki blok kardeş alt alta; toplam yükseklik ikisinin toplamı
  - `margin-top` konumu kaydırıyor
  - iki kardeş arasında margin collapsing (Taffy'nin block layout'u bunu
    yapıyor mu burada ölçülür; yapmıyorsa bu M1'in akış layout'u kararına veri
    olur ve Yürütme Notları'na yazılır)
- [x] **Step 4:** Commit: `feat(layout): block layout on the DOM with Taffy`.

---

### Task 5: Paragraf yaprağı — Parley

**Tam IFC değil.** Satırlar arası span kırılması, satır içi görseller ve karışık
stiller M1'de.

**Files:**
- Create: `crates/erk-renderer/src/layout/text.rs`, `assets/fonts/`

- [x] **Step 1: Gömülü yazı tipi.** Türkçe glifleri içeren OFL lisanslı bir yazı
  tipi (ör. Noto Sans Regular) `assets/fonts/` altına, lisans dosyasıyla birlikte
  eklenir. Dosyanın indirilmesi için kullanıcıdan onay alınır. Altın testler
  yalnızca bu yazı tipini kullanır.
- [x] **Step 2:** Çocukları yalnızca metin olan blok (M0'da inline öğeler
  metinleri ebeveynin stiliyle birleştirilir), Taffy'de **ölçüm fonksiyonlu bir
  yapraktır**. Ölçüm fonksiyonu Parley ile metni ebeveynin hesaplanmış yazı
  tipi, boyutu ve satır yüksekliğiyle şekillendirir, verilen genişlikte satırlara
  böler ve `(genişlik, yükseklik)` döner.
- [x] **Step 3:** Parley layout'u `NodeId` ile önbelleğe alınır; boyama aynı
  layout'u kullanır, yeniden şekillendirmez.
- [x] **Step 4: Testler:**
  - uzun paragraf dar genişlikte birden fazla satıra bölünüyor, yüksekliği satır
    sayısıyla orantılı
  - `İstanbul` şekillendiriliyor ve her karakter için bir glif var (gömülü yazı
    tipinde eksik glif yok)
- [x] **Step 5:** Commit: `feat(layout): shape paragraphs with Parley as Taffy
  leaves`.

---

### Task 6: Display list ve boyama

**Files:**
- Create: `crates/erk-renderer/src/display.rs`, `src/paint.rs`,
  `tests/golden.rs`, `tests/golden/merhaba.png`

- [x] **Step 1:** `DisplayItem`: `Rect { rect, color }` (arka plan) ve
  `GlyphRun { font, size, color, glyphs }`. Layout ağacı gezilerek kurulur:
  önce arka planlar, sonra metin. Display list metin olarak da dökülebilir
  (hata ayıklama ve ileride reftest için).
- [x] **Step 2:** `vello_cpu` 0.2'nin API'si docs.rs'ten okunur (render bağlamı,
  paint ayarı, dikdörtgen doldurma, glif çizme, pixmap'e boyama). Display list
  sırayla boyanır; tek iş parçacığı.
- [x] **Step 3:** PNG'ye yazma (`png` crate'i).
- [x] **Step 4: Altın test.** `tests/golden.rs` `examples/merhaba.html`'i
  800×600'de boyar ve `tests/golden/merhaba.png` ile piksel piksel karşılaştırır.
  Uyuşmazlıkta gerçek çıktı `target/golden-actual/` altına yazılır ki fark
  incelenebilsin. İlk referans elle incelenip commit'lenir.
- [x] **Step 5: Kasıtlı ihlal:** Varsayılan metin rengini değiştir → test kırmızı.
  Geri al.
- [x] **Step 6:** Commit: `feat(paint): display list painted with vello_cpu`.

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

### Task 1 tamamlandı (2026-09-25)

Doğrulanan araç sürümleri: rustup 1.29.1 (winget `Rustlang.Rustup`), rustc ve
cargo 1.98.1, Python 3.14.4, Visual Studio Community 2022 (C++ araçları kurulu,
ayrıca Build Tools gerekmedi).

| Plan ne diyordu | Gerçek |
|---|---|
| `rustup toolchain install` argümansız, dosyadaki toolchain'i kurar mı? (doğrulanacak) | **Kuruyor.** rustup 1.29.1'de `rust-toolchain.toml`'daki `1.98.1`'i indirdi ve "overridden by rust-toolchain.toml" diye etkinleştirdi. CI adımı buna dayanıyor. |
| Lint devralma kontrolü `rust-checks` job'ında bir adım | Ayrı bir `guards` job'ı (ubuntu). Statik kontroller işletim sistemine bağlı değil, iki kez koşmalarına gerek yok; sonraki `cargo tree` muhafızları da buraya eklenecek. |
| — | **Edition 2024 ve `resolver = "3"`** eklendi (plan edition'dan söz etmiyordu). Boş crate'lerde bedeli sıfır, kod yazıldıktan sonra geçiş bir iş. Resolver 3 MSRV'ye duyarlı: `rust-version`'dan yeni sürüm isteyen bağımlılığı seçmez. |
| — | `rust-version = "1.98"` workspace'te, her crate devralıyor. |

**CI kontrol adları değişti:** matris yüzünden kontroller artık
`rust-checks (ubuntu-latest)`, `rust-checks (windows-latest)` ve `guards`.
Branch protection'da zorunlu check olarak bu üçü seçilmeli; eski `rust-checks`
adı artık raporlanmayacak.

Kasıtlı ihlaller:

- `erk-dom`'a `unsafe {}` → `error: usage of an unsafe block`, not olarak
  "requested on the command line with `-F unsafe-code`" (workspace lint'i
  derleyiciye `-F` olarak geçiyor). Geri alınınca derleme yeşil.
- `erk-network`'ten `[lints]` bloğu silindi → `guards` betiği "does not inherit
  [workspace.lints]" ile çıkış kodu 1. Geri alınınca 0.

`Cargo.lock` ilk kez işlendi (Erk bir uygulama; bkz. doğrulama §2).

### Task 2 tamamlandı (2026-09-25)

| Plan ne diyordu | Gerçek |
|---|---|
| `html5ever::QuirksMode` | Kök düzeyde yok; `html5ever::tree_builder::QuirksMode`. |
| `TreeSink::ElemName` ilişkili tipi docs'tan okunacak | markup5ever 0.39 `Ref<'_, QualName>` için `ElemName` sağlıyor. Belge düzeyinde tek `RefCell` ve `Ref::map` yetiyor. Tree builder bir isim `Ref`'ini tutarken değiştirici bir çağrı yapsaydı çakışma paniği verirdi; adoption agency ve foster parenting testleri bunu tetiklemedi. |
| Metin ve öznitelik tipi belirtilmemişti | `String`, `StrTendril` değil. Tendril `Send` değil; Stylo'nun ileride paralel gezinmesi DOM'un iş parçacıkları arasında paylaşılmasını istiyor. `html5ever::Attribute` bu yüzden kendi `Attribute` tipimize çevriliyor. |
| — | Yan tablolar için `NodeId::index()` ve `Document::capacity_hint()` eklendi (Task 3 ve 4'te Stylo ve Taffy verisi için). |
| — | Ayrılan düğümler arenadan silinmiyor; silme API'si yok. Düğüm sahipliği M4'te JS ile birlikte karar verilecek (tasarım §5.3). |
| — | Ayrıştırma hataları yok sayılıyor: tree builder spesifikasyonun kurtarmasını zaten uyguluyor. |

**Planın öngörmediği muhafız:** clippy'nin `disallowed_types` lint'i
`#[allow(clippy::disallowed_types)]` ile susturulabiliyor. `guards` job'ına
`erk-dom` içinde bu ifadenin geçmediğini kontrol eden bir adım eklendi.

Kasıtlı ihlaller:

- `erk-dom`'a `pub struct Violation(pub std::rc::Rc<u8>);` → clippy: "use of a
  disallowed type `std::rc::Rc`". Geri alınınca yeşil.
- `erk-dom`'a `erk-network` bağımlılığı → yaprak betiği `erk-network`'ü basıp 1
  ile çıktı. Geri alınınca 0.
- `lib.rs`'e `#[allow(clippy::disallowed_types)]` → betik satırı basıp 1 ile
  çıktı. Geri alınınca 0.
- **Testin kendisi de sınandı:** `append_before_sibling`'deki metin birleştirme
  kapatılınca foster parenting testi `"a","b"` ≠ `"ab"` ile kırmızı.

html5ever 0.39.0'ın çözdüğü atom crate'leri `web_atoms` 0.2.6 ve `string_cache`
0.9.0; Stylo'nun beklediği hat bu.

### Task 3 tamamlandı (2026-09-25)

Stylo 0.20 tek başına derlendi: soğuk derleme 3 dk 18 sn, `target/` ~1 GB.
Python 3.14 PATH'teki `python.exe` ile kendiliğinden bulundu, `PYTHON3`
gerekmedi. LLVM istenmedi.

| Plan ne diyordu | Gerçek |
|---|---|
| Adaptör `erk-renderer/src/style/` altında | **Ayrı crate: `erk-style`.** Stylo'nun `TElement`'i beş metodu `unsafe fn` olarak tanımlıyor (`ensure_data`, `clear_data`, `set_dirty_descendants`, `unset_dirty_descendants`, `set_handled_snapshot`). Bir deneyle doğrulandı: gövde boş olsa bile `unsafe fn` bir trait metodunu uygulamak "implementation of an `unsafe` method" hatası veriyor, `forbid` altında öğe düzeyinde `allow` ise E0453 ile reddediliyor. `forbid` altındaki bir crate Stylo'yu hiç bağlayamaz. `erk-style` `deny` seviyesinde ve izin yalnızca bu beş imzada; gövdeler yan tabloya güvenli çağrılar. `erk-renderer` `forbid`'de kalıyor. |
| Tutamak `ErkNode { doc, side, id }` | **Stylo, `TElement` tipinin tam bir işaretçi genişliğinde olmasını şart koşuyor.** Stil paylaşım önbelleği tipi `transmute` ile siliyor ve boyutları yalnızca çalışma zamanında `assert` ediyor (`sharing/mod.rs:611`). 16 baytlık tutamakla altı testin altısı da burada düştü (9744 ≠ 9488). Tutamak artık `ErkNode(&StyledNode)`: her yan tablo kaydı kendi `NodeId`'sini ve ağaca bir referansı tutuyor. Ağaç ile kayıtlar birbirini gösteriyor; kayıtlar ağaç oluştuktan sonra bir `OnceLock` ile yerleştiriliyor, güvenli kodda. Boyut artık **derleme zamanında** doğrulanıyor (`const assert`). |
| Yan tablo sapması (Blitz'ten), trait imzası izin vermezse Blitz modeline dönülecek | **Tuttu.** Blitz modeli, düğüme ağacını gösteren ham bir işaretçi koymayı gerektirirdi (Blitz öyle yapıyor), yani `erk-dom`'a `unsafe` sokmak. Yan tablo `erk-dom`'u hem yaprak hem `forbid` tutuyor. |
| UA stil sayfası Blitz'ten alınacak | Blitz'inki Firefox'un `html.css`'inden türetilmiş ve **MPL-2.0**. Kopyalansaydı `erk-style`'ın lisansı "MIT OR Apache-2.0" kalamazdı. HTML Standardı'nın "Rendering" bölümünden kendi küçük stil sayfamız yazıldı (`erk-style/src/ua.css`); M1'de genişler. |
| — | Yazı tipi ölçümleri (`ex`, `ch` birimleri için) Task 5'e kadar yazı tipi boyutunun sabit oranları. |
| — | `Styles` yalnızca hesaplanmış değerleri dışarı taşıyor; Stylo'nun `ElementData`'sı kalıcı değil, her çağrı tam yeniden stil. Artımlı stil (M5) kalıcı bir ağaç isteyecek. |
| — | Sunumsal öznitelikler (presentational hints) şimdilik yalnızca `bgcolor` ve `align`; Blitz çok daha fazlasını eşliyor, ilgili elemanlarla gelecek. |
| — | Stylo 0.20'nin `TElement` metot listesi Blitz'inkiyle birebir örtüştü; eksik ya da fazla metot hatası çıkmadı. |

Testler (`erk-style/tests/computed.rs`, 6 test): yazar stil sayfası, UA
stil sayfası (`h1` blok ve 32px), `style` özniteliği, kalıtım, sınıf ve id
seçicileri, `display: none` altındaki elemanların stillenmemesi.

Kasıtlı ihlaller:

- `erk-style`'da `unsafe_code = "deny"` → `"allow"`: lint devralma betiği
  "is a lint exception but does not deny unsafe_code" ile 1. Geri alınınca 0.
- `erk-style`'a altıncı bir `#[allow(unsafe_code)]`: yüzey betiği `allows=6`
  ile 1. Geri alınınca `allows=5 blocks=0`, 0.
- `erk-dom`'da `html5ever = "=0.40.1"`: `cargo tree -d` hem `web_atoms`
  0.2.6/0.3.0 hem `string_cache` 0.9.0/0.11.0 gösterdi, muhafız 1. Derleme de
  `has_local_name`, `has_namespace`, `local_name` üzerinde E0053 ile kırıldı;
  muhafız sebebi daha açık söylüyor. Geri alınınca 0.

`stylo_taffy` henüz eklenmedi; Task 4'te gelecek.

### Task 4 tamamlandı (2026-09-25)

| Plan ne diyordu | Gerçek |
|---|---|
| `stylo_taffy` MPL-2.0, yalnızca bağımlılık olarak | `stylo_taffy` 0.3.0-beta.2'nin lisansı **"MIT OR Apache-2.0 OR MPL-2.0"**; Erk onu MIT OR Apache-2.0 altında kullanıyor. Araştırma notundaki "MPL" eksikti. Sürüm blitz-dom 0.3.0-beta.2'ninkiyle aynı (Stylo 0.20 ile eşleşiyor). |
| — | **`calc()` için Blitz'in yolu `unsafe` istiyordu.** `stylo_taffy`, `calc()` değerlerini Taffy'ye Stylo'nun `CalcLengthPercentage`'ine işaret eden ham işaretçiler olarak geçiriyor ve bu yüzden Taffy'nin `calc` özelliğini zorunlu kılıyor. Taffy çözümleme için işaretçiyi `resolve_calc_value` ile geri veriyor; Blitz onu `unsafe` ile izliyor. `erk-renderer` `forbid` altında. Çözüm (`layout/calc.rs`): layout ağacı kurulurken her düğümün `calc()` değerleri adresleriyle bir tabloya kopyalanıyor, Taffy'nin işaretçisi yalnızca **anahtar** olarak kullanılıyor, hiç izlenmiyor. Kapsam: boyutlar, min/max boyutlar, margin, padding ve inset. Tabloda olmayan bir adres debug derlemede `debug_assert` ile patlıyor, release'de 0'a düşüyor (Taffy'nin çözücüsüz davranışı). Grid track'leri ve `gap` henüz tabloda değil. |
| Margin collapsing ölçülecek | **Taffy yapıyor:** kardeşler arasında `margin-bottom: 20px` ve `margin-top: 30px` → aradaki boşluk 30 (CSS 2 §8.3.1), 50 değil. |
| Layout ağacı | Erk'in DOM'u layout ağacı; Taffy'nin düğüm başına durumu (`Style`, `Cache`, yuvarlanmamış ve son `Layout`) `NodeId::index()` ile bir yan tabloda. Blok, flow-root, flex ve grid bağlanmış; M0 testleri blok. Belge düğümü ilk kapsayıcı bloğun kutusu. |
| Testler crate dışından | Layout modülü `pub(crate)`, testler modül içinde. Sebep: `erk-renderer`'ın genel yüzeyi Task 7'de yalnızca iş parçacığı ve mesajlar olacak; kabuk DOM tiplerini görmemeli. |

Testler (`erk-renderer/src/layout/tests.rs`, 8 test): açık genişlik, varsayılan
genişliğin kapsayıcıyı doldurması, blokların alt alta dizilmesi ve ebeveyn
yüksekliği, `margin-top`, margin collapsing, `display: none`, `calc(50% - 20px)`,
padding ve border'ın border-box'ı genişletmesi.

**Testin kendisi de sınandı:** `resolve_calc_value` geçici olarak hep 0 dönecek
şekilde değiştirilince `calc()` testi `0.0` ≠ `380.0` ile kırmızı.

Bu görev yeni bir mimari kural getirmedi. `erk-renderer`'ın işaretçi
izlememesini `forbid(unsafe_code)` zaten zorluyor.

### Task 5 tamamlandı (2026-09-25)

Yazı tipleri kullanıcının onayıyla Noto'nun resmi deposundan indirildi
(`notofonts/notofonts.github.io`, hinted TTF): `NotoSans-Regular.ttf`
621 572 bayt, `NotoSans-Bold.ttf` 631 484 bayt, lisans
`notofonts/latin-greek-cyrillic` deposundan `OFL.txt`. `erk-renderer`'ın
lisans ifadesi bu yüzden `(MIT OR Apache-2.0) AND OFL-1.1`.

| Plan ne diyordu | Gerçek |
|---|---|
| Tek yazı tipi | **Regular ve Bold.** Yalnızca Regular olsaydı fontique kalınlığı sentezlerdi; sentetik kalınlık glif genişliklerini değiştirmiyor. Test: Bold dosyası kayıttan çıkarılınca "Bold, Regular'dan geniş" testi kırılıyor. |
| Çocukları yalnızca metin olan blok bir yaprak | Kapsam biraz genişledi: çocukları **metin ve satır içi elemanlar** olan blok bir paragraf yaprağı; `<b>` gibi elemanların metni paragrafa katılıyor (stilleri değil — tüm paragraf bloğun stiliyle şekilleniyor). Blok çocuklarla karışık metin bırakılıyor; anonim blok kutuları M1'de. |
| Yazı tipi ölçümleri Task 5'te Parley'e bağlanacak | Parley'e değil **skrifa'ya** (Parley'in kullandığı sürüm, 0.44.0). `erk-style` yazı tipi bilmiyor; `StyleEngine::with_font_metrics` ile sağlayıcıyı dışarıdan alıyor, `erk-renderer` gömülü Noto Sans'tan okuyan `EmbeddedFontMetrics`'i veriyor. Sabit oranlı sağlayıcı yalnızca `erk-style`'ın kendi testlerinde. Test: `10ex` = 86px, `10ch` = 92px (sabit oranlarla 80 olurdu). |
| — | Sistem yazı tipleri kapalı (`parley` `default-features = false`): ölçüm ve çizim her makinede aynı. CSS `font-family` henüz okunmuyor, her şey Noto Sans. |
| — | `line-height: normal` → Parley `MetricsRelative(1.0)`: yazı tipinin kendi satır aralığı. 16px Noto Sans'ta bir satır ~21.8px. |
| — | Beyaz boşluk `white-space: normal` gibi çöküyor ve kenarlardan kırpılıyor; `pre` ve diğer kipler M1'de. |
| — | Son layout'tan sonra her paragraf kesin içerik genişliğinde bir kez daha şekilleniyor ve boyama için `Layouts::text` ile saklanıyor. |
| — | `LayoutTree` artık bir `&mut TextEngine` tutuyor; Taffy'nin GAT'leri bu yüzden uygulamada da `where Self: 'a` istiyor. |

Testler: `text.rs` içinde 5 (beyaz boşluk, Türkçe harflerin hepsinin glifi
var, dar genişlikte satır kırma, Bold yüzü, min-content = en uzun kelime),
layout testlerine 6 (paragrafın bir satır yüksekliği, dar kapsayıcıda
kırılma, paragraflar arasında çöken margin, satır içi elemanların metni,
başlığın büyüklüğü, gömülü fonttan `ex`/`ch`).

**Testin kendisi de sınandı:** Bold kayıttan çıkarılınca Bold testi kırmızı.

### Task 6 tamamlandı (2026-09-25)

İlk pikseller: `examples/merhaba.html` (Task 8'in kabul sayfası, burada
oluşturuldu) 800×600'de doğru çiziliyor. Altın görüntü elle incelendi:
kalın kırmızı başlık, iki satıra kırılan paragraf, 16px iç boşluklu beyaz
kutu, eksiksiz Türkçe karakterler, body arka planının tuvale yayılması.

| Plan ne diyordu | Gerçek |
|---|---|
| `vello_cpu` tek iş parçacığında | Tek iş parçacığı yetmiyor: `vello_cpu` çalıştığı işlemcinin SIMD seviyesini (SSE/AVX2/NEON) kendisi seçiyor ve seviyeler farklı yuvarlayabiliyor. Boyama **`Level::baseline()`** ile sabit; altın görüntüler Windows ve Linux CI'da aynı çıkmalı. Bedeli hız; hız M2'deki GPU yolunun işi. |
| Display list: arka planlar ve glif çalışmaları | Aynen. Ayrıca **tuval arka planı** (CSS 2 §14.2): kök elemanın, o yoksa body'nin arka planı tüm tuvali boyar ve kendisi ikinci kez boyanmaz. Kenarlıklar henüz yok. |
| Display list metin olarak dökülebilir | `DisplayList::dump`: satır başına bir öğe (`rect x y wxh #rrggbbaa`, `glyphs x y 16px #... "metin"`). Altın test kırıldığında gerçek PNG ile birlikte `target/golden-actual/` altına yazılıyor; "ne kaydı" sorusu görüntüye bakmadan cevaplanıyor. |
| — | `erk-renderer`'ın genel yüzeyi: `render_html(html, genişlik, yükseklik) -> Frame` (`rgba`, `to_png`, `display_list`). HTML metni girer, piksel çıkar; DOM, stil ve layout tipleri crate içinde kalır. |
| — | Altın görüntü yalnızca `ERK_BLESS=1` ile yeniden yazılır. Karşılaştırma PNG baytlarıyla değil çözülmüş piksellerle; kodlayıcı sürümü değişse de test kırılmaz. |
| — | Paragraf metni artık `Layouts` içinde şekillenmiş layout'la birlikte (`ShapedText`) duruyor; dökümde glif çalışmasının kaynak metni gösteriliyor. |

Testler: `tests/golden.rs` (altın görüntü, iki çizimin bayt bayt aynı olması).

Kasıtlı ihlal: glif hinting'i kapatıldı (`.hint(false)`) → altın test
kırmızı, gerçek çıktı ve döküm `target/golden-actual/` altında. Geri alınınca
yeşil.
