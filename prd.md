# PRD: Browser Desktop Rust

## 1. Ringkasan

Project ini bertujuan membangun browser desktop yang cepat, ringan, dan tetap familier untuk pengguna umum. Produk diposisikan sebagai browser modern dengan fokus pada startup cepat, UI responsif, konsumsi resource yang efisien, dan pengalaman browsing inti yang bersih.

Visi jangka panjangnya cukup ambisius, tetapi versi awal harus realistis untuk dibangun oleh solo maker atau small team. Karena itu, MVP diarahkan sebagai browser shell berbasis Rust yang menggunakan rendering engine existing, bukan membangun browser engine sendiri dari nol.

Dokumen ini dimaksudkan sebagai dasar kerja untuk founder, product, dan engineering agar bisa langsung memecah project menjadi milestone, backlog, dan prioritas implementasi.

## 2. Latar Belakang dan Problem Statement

Browser modern sering terasa semakin berat. Waktu startup melambat, konsumsi memori meningkat, dan fitur bertambah jauh lebih cepat daripada kebutuhan mayoritas pengguna. Banyak user sebenarnya hanya membutuhkan browser yang:

- cepat dibuka
- responsif saat dipakai
- stabil untuk browsing harian
- cukup sederhana untuk dipahami tanpa kurva belajar baru

Masalah yang ingin diselesaikan project ini adalah menghadirkan browser desktop yang lebih ringan dan lebih cepat, tanpa memaksa user mempelajari pola interaksi baru atau kehilangan fungsi browsing inti yang mereka anggap standar.

## 3. Visi Produk

Membangun browser desktop modern yang:

- terasa ringan sejak pertama dibuka
- menjaga interaksi utama tetap cepat dan minim friksi
- menyediakan pengalaman browsing yang familier untuk pengguna umum
- menjadi fondasi yang sehat untuk pengembangan fitur lebih lanjut di masa depan

Positioning produk:

- bukan browser eksperimental untuk niche power user
- bukan browser privacy-first yang penuh pengaturan kompleks
- bukan browser dengan ambisi langsung menyaingi seluruh fitur Chrome atau Firefox
- merupakan browser yang menekankan performa, kesederhanaan, dan pengalaman inti yang rapi

## 4. Target User

Target utama versi pertama adalah pengguna umum desktop yang:

- menggunakan browser untuk browsing harian
- membuka beberapa tab sekaligus
- mengakses situs umum seperti search, docs, media, e-commerce, dan layanan login standar
- ingin pengalaman yang terasa cepat dan ringan
- tidak ingin belajar UX baru yang terlalu asing

Karakteristik target user:

- bukan technical enthusiast yang mengharapkan kontrol tingkat lanjut sejak hari pertama
- tidak membutuhkan extension ecosystem pada MVP
- tetap mengharapkan perilaku browser yang familier seperti address bar, tab, history, bookmark, dan download

## 5. Goals

### Product Goals

- Menyediakan browser desktop yang startup-nya terasa cepat.
- Menjaga navigasi utama tetap responsif untuk use case harian.
- Menyediakan alur browsing yang familier bagi pengguna umum.
- Mendukung multi-tab dasar dengan perilaku yang stabil.
- Menjaga cakupan MVP tetap realistis untuk solo maker atau small team.
- Mencapai baseline usability di macOS, Windows, dan Linux.

### Success Criteria

- Pengguna bisa membuka aplikasi dan mulai browsing tanpa onboarding khusus.
- User flow utama berjalan tanpa kebingungan: membuka URL, search, pindah tab, bookmark, history, dan download dasar.
- Performa awal dan respons UI terasa lebih ringan dibanding persepsi umum terhadap browser mainstream yang berat.
- Struktur produk siap dikembangkan ke fase berikutnya tanpa perlu rewrite arsitektur inti.

## 6. Non-Goals

Fitur dan sasaran berikut tidak termasuk dalam MVP:

- membangun browser engine sendiri
- membangun JavaScript engine sendiri
- extension marketplace atau extension compatibility penuh
- account sync lintas device
- profile management kompleks
- devtools lanjutan
- sistem sandbox/security penuh setara browser besar
- optimasi media/DRM tingkat lanjut
- UI eksperimental yang mengubah total kebiasaan browsing user

## 7. Prinsip Produk

- **Cepat lebih penting daripada ramai fitur.** Fitur yang tidak memperkuat browsing inti tidak menjadi prioritas awal.
- **Familier lebih penting daripada unik.** Pengguna umum harus langsung paham cara memakai produk tanpa adaptasi besar.
- **Ringan secara teknis dan mental.** Aplikasi tidak hanya hemat resource, tetapi juga terasa sederhana saat digunakan.
- **Realistis secara implementasi.** Keputusan teknis harus mendukung delivery MVP yang usable, bukan mengejar ambisi riset terlalu dini.

## 8. Cakupan MVP

MVP harus mencakup kemampuan berikut:

- address bar untuk URL dan search
- back, forward, reload
- multi-tab dasar
- loading state dan error state dasar
- bookmark
- history
- download dasar
- settings dasar
- session behavior yang stabil pada penggunaan normal

### Detail Scope MVP

#### 8.1 Navigation

- User dapat memasukkan URL langsung dari address bar.
- User dapat mengetik query yang diperlakukan sebagai search.
- Browser menyediakan aksi back, forward, dan reload yang konsisten.
- Browser menampilkan status loading dasar saat halaman dimuat.

#### 8.2 Tabs

- User dapat membuka tab baru.
- User dapat berpindah tab.
- User dapat menutup tab.
- Browser menjaga state tab tetap konsisten selama satu sesi pemakaian.

#### 8.3 Bookmark

- User dapat menyimpan halaman aktif ke bookmark.
- User dapat melihat daftar bookmark.
- User dapat membuka kembali bookmark dari UI yang sederhana.

#### 8.4 History

- Browser menyimpan riwayat halaman yang dikunjungi.
- User dapat melihat history dan membuka kembali halaman sebelumnya.
- History cukup mendukung kebutuhan revisit halaman, tanpa fitur manajemen lanjutan.

#### 8.5 Downloads

- Browser mendukung download file umum.
- User dapat melihat status dasar download.
- Browser menampilkan informasi dasar bila download gagal.

#### 8.6 Settings

- Browser menyediakan preferensi dasar yang relevan untuk penggunaan awal.
- Settings difokuskan pada hal-hal minimum yang mempengaruhi pengalaman dasar, bukan kontrol tingkat lanjut.

## 9. User Flows Utama

### 9.1 First Launch

1. User membuka aplikasi.
2. Browser tampil cepat dengan state awal yang jelas.
3. User langsung melihat address bar dan dapat mulai browsing.

### 9.2 Buka URL atau Search

1. User mengetik alamat situs atau kata kunci di address bar.
2. Browser mengenali input sebagai URL atau search query.
3. Halaman dimuat dan user menerima feedback loading yang jelas.

### 9.3 Multi-Tab Dasar

1. User membuka tab baru.
2. User berpindah antar tab.
3. User menutup satu atau beberapa tab.
4. Browser menjaga tab strip dan fokus aktif tetap konsisten.

### 9.4 Revisit dari History

1. User membuka halaman history.
2. User memilih halaman yang pernah dibuka.
3. Browser membuka kembali halaman tersebut dengan alur yang sederhana.

### 9.5 Simpan Bookmark

1. User membuka halaman tertentu.
2. User menyimpan halaman ke bookmark.
3. User dapat kembali membuka halaman itu dari daftar bookmark.

### 9.6 Download File

1. User memulai download file dari halaman web.
2. Browser menampilkan progress atau status dasar.
3. User dapat mengetahui apakah download selesai, gagal, atau masih berjalan.

## 10. Antarmuka Produk

Permukaan utama yang harus terlihat dan dipahami user:

- window utama
- toolbar/navigation controls
- address bar
- tab strip
- area konten halaman
- surface bookmark
- surface history
- surface download
- settings dasar

Tujuan desain antarmuka pada MVP adalah kejelasan dan familiaritas. UI tidak perlu tampil eksperimental selama itu membantu mempercepat adopsi awal dan meminimalkan kebingungan user.

## 11. Entitas Data Minimum

PRD ini mengasumsikan kebutuhan entitas data minimum berikut:

### 11.1 Tab

- identifier tab
- judul halaman
- URL aktif
- status loading
- state aktif/non-aktif

### 11.2 History Entry

- URL
- judul halaman
- timestamp kunjungan

### 11.3 Bookmark Item

- URL
- judul
- timestamp penyimpanan

### 11.4 Download Item

- identifier download
- nama file
- status
- lokasi file atau referensi hasil

### 11.5 Preferences / Settings

- konfigurasi dasar yang mempengaruhi perilaku browser
- nilai-nilai sederhana yang perlu dipertahankan antar sesi

## 12. Arah Teknis Produk

Arsitektur MVP harus dinyatakan secara eksplisit sebagai:

`Rust app shell + existing rendering engine`

Artinya:

- aplikasi inti ditulis dengan Rust
- state aplikasi, window behavior, navigasi, dan fitur browser shell dikelola oleh app layer
- rendering halaman web menggunakan engine existing/platform yang sudah tersedia

Keputusan ini penting agar scope project tetap realistis dan tidak bergeser menjadi proyek membangun browser engine dari nol.

## 13. Batasan Cross-Platform

Target produk adalah desktop cross-platform:

- macOS
- Windows
- Linux

Namun, implementasi MVP boleh menggunakan abstraction di atas engine platform/existing yang memiliki capability berbeda di tiap OS. Konsekuensinya:

- feature parity lintas platform adalah target, bukan jaminan mutlak pada tahap paling awal
- perilaku media, sandbox, download, dan kompatibilitas situs dapat berbeda secara terbatas di tiap platform
- prioritas engineering adalah menjaga pengalaman inti tetap konsisten, meskipun detail tertentu berbeda per OS

## 14. Kebutuhan Non-Fungsional

### 14.1 Performa

- aplikasi harus terasa cepat saat startup
- interaksi UI utama harus responsif
- perpindahan tab dan navigasi utama harus terasa ringan

### 14.2 Resource Efficiency

- penggunaan memori harus masuk akal untuk skenario jumlah tab ringan hingga menengah
- aplikasi tidak boleh menunjukkan overhead yang tidak proporsional dibanding ruang lingkup fiturnya

### 14.3 Stabilitas

- sesi browsing normal harus stabil
- crash handling harus jelas
- relaunch setelah gangguan harus tetap bisa dilakukan dengan aman

### 14.4 Safety dan Fallback

- bila ada situs atau fitur web yang tidak didukung dengan baik oleh engine, browser harus memiliki perilaku fallback yang jelas
- error state harus dapat dipahami user

## 15. Success Metrics

Metrik awal yang perlu dipakai untuk mengevaluasi produk:

- waktu startup aplikasi
- crash-free session rate
- time-to-first-page
- responsiveness saat navigasi dan pindah tab
- keberhasilan penyelesaian task dasar
- retensi pengguna awal

### Definisi keberhasilan task dasar

Task dasar dianggap berhasil bila user bisa:

- membuka browser dan mulai browsing
- melakukan URL/search dari address bar
- menggunakan back/forward/reload
- membuka dan menutup tab
- menyimpan serta membuka bookmark
- melihat history
- menyelesaikan download file sederhana

## 16. Risiko Utama

### 16.1 Perbedaan Capability Engine antar OS

Rendering engine existing bisa memiliki perilaku berbeda di macOS, Windows, dan Linux. Risiko ini dapat mempengaruhi konsistensi fitur dan kompatibilitas situs.

### 16.2 Media dan DRM Compatibility

Beberapa situs media atau protected content mungkin tidak berjalan konsisten pada MVP. Ini tidak menjadi blocker untuk browsing inti, tetapi harus diakui sejak awal.

### 16.3 Kompleksitas Sandbox dan Security Penuh

Browser modern memiliki kebutuhan security yang sangat kompleks. MVP tidak menargetkan kesetaraan penuh dengan browser besar pada area ini.

### 16.4 Distribusi dan Update Desktop

Distribusi lintas platform dan mekanisme update dapat menambah kompleksitas operasional, terutama jika target platform dikembangkan paralel sejak awal.

## 17. Roadmap Produk

### Fase 1: Prototype Browser Shell

Fokus:

- validasi arsitektur Rust app shell
- integrasi rendering engine existing
- single-window browsing
- basic navigation

Output utama:

- prototype yang bisa membuka halaman web dan menjalankan alur browsing dasar

### Fase 2: MVP Essential Browser

Fokus:

- multi-tab dasar
- bookmark
- history
- download dasar
- settings dasar
- stabilitas dan baseline cross-platform

Output utama:

- browser usable untuk browsing harian ringan dengan pengalaman yang familier

### Fase 3: Post-MVP

Fokus:

- session restore
- privacy improvements
- peningkatan reliabilitas dan polish lintas platform
- eksplorasi diferensiasi lanjutan berdasarkan feedback user

Output utama:

- produk yang lebih matang untuk evaluasi distribusi lebih luas

## 18. Test Plan

### 18.1 First Run

- aplikasi terbuka cepat
- state awal jelas
- address bar langsung terlihat dan siap dipakai

### 18.2 Browsing Inti

- user memasukkan URL atau search query
- halaman termuat dengan feedback loading yang jelas
- back/forward/reload bekerja tanpa perilaku membingungkan

### 18.3 Tabs

- user membuka beberapa tab
- user berpindah tab
- user menutup tab
- state aktif tetap konsisten

### 18.4 Persistence

- bookmark tersimpan setelah relaunch
- history tetap tersedia setelah relaunch
- settings dasar tetap dipertahankan antar sesi

### 18.5 Downloads

- file umum dapat diunduh
- progress atau status dasar terlihat
- error state dapat dipahami

### 18.6 Failure Scenarios

- URL invalid ditangani dengan jelas
- halaman gagal dimuat menampilkan state yang dapat dipahami
- relaunch setelah crash atau gangguan tetap bisa dilakukan
- perilaku unsupported site tidak membuat browser terasa rusak total

### 18.7 Non-Functional Validation

- startup diuji di macOS, Windows, dan Linux
- responsiveness dinilai pada navigasi dan perpindahan tab
- stabilitas diuji pada penggunaan harian ringan
- penggunaan resource dipantau pada jumlah tab ringan hingga menengah

## 19. Asumsi dan Default

- Dokumen ini ditulis dalam bahasa Indonesia dengan tone praktis.
- Audience utama adalah founder, product, dan engineering kecil.
- MVP mengutamakan familiaritas dan performa, bukan eksperimen UX besar.
- Jalur implementasi diasumsikan realistis untuk solo maker atau small team.
- Riset engine internal hanya relevan sebagai visi jangka panjang, bukan bagian dari delivery MVP.

## 20. Ringkasan Eksekutif

Project ini layak dibangun bila dijaga tetap fokus: browser desktop yang cepat, ringan, dan familier, dengan Rust sebagai fondasi aplikasi inti dan engine existing sebagai lapisan rendering web. PRD ini sengaja menghindari ambisi yang terlalu luas di fase awal agar tim dapat bergerak cepat menuju prototype, lalu berkembang ke MVP yang usable dan terukur.
