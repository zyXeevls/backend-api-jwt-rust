# Backend API JWT (Rust + Axum + SQLx)

Backend API berbasis Rust untuk kebutuhan autentikasi JWT pada stack fullstack Rust + Vue.

## Ringkasan

Proyek ini menggunakan:

- **Axum** untuk HTTP server dan routing
- **SQLx** untuk koneksi database PostgreSQL
- **JWT (`jsonwebtoken`)** untuk token auth
- **bcrypt** untuk hashing password
- **dotenvy** untuk membaca konfigurasi dari `.env`

Saat ini proyek sudah bisa:

- Menjalankan server Axum
- Terhubung ke PostgreSQL
- Menjalankan migrasi tabel `users`
- Build/check tanpa error (`cargo check`)

## Teknologi

- Rust (Edition 2024)
- Axum `0.8`
- SQLx `0.8` (PostgreSQL runtime Tokio)
- Tokio
- JsonWebToken (`rust_crypto` backend)
- Validator

## Struktur Proyek

```text
src/
	main.rs
	config/
		database.rs
	handlers/
		register.handler.rs
	middlewares/
		auth.middleware.rs
	models/
		user.rs
	routes/
		auth.routes.rs
	schemas/
		register.schema.rs
	utils/
		jwt.rs
		response.rs

migrations/
	20260303074236_create_users_table.sql
```

## Prasyarat

Pastikan sudah terpasang:

1. Rust dan Cargo
2. PostgreSQL
3. (Opsional, direkomendasikan) `sqlx-cli` untuk migrasi

Install `sqlx-cli`:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

## Konfigurasi Environment

File `.env`:

```env
APP_PORT=3000
DATABASE_URL=postgresql://postgres@localhost:5432/db_rust_vue?schema=public
JWT_SECRET=belajar_rust_vue_jwt_secret_key
```

Penjelasan variabel:

- `APP_PORT`: port server (default fallback di kode: `3001` bila env tidak valid/tidak ada)
- `DATABASE_URL`: koneksi ke PostgreSQL
- `JWT_SECRET`: secret untuk sign/verify JWT

## Menjalankan Proyek

1. **Clone dan masuk direktori proyek**
2. **Pastikan database tujuan sudah ada**
3. **Jalankan migrasi**

```bash
sqlx migrate run
```

4. **Jalankan aplikasi**

```bash
cargo run
```

Server akan aktif di:

```text
http://127.0.0.1:<APP_PORT>
```

## Migrasi Database

Migrasi saat ini membuat tabel `users`:

- `id` (BIGINT, identity, primary key)
- `name` (varchar)
- `email` (varchar, unique)
- `password` (varchar)
- `created_at` (timestamp default `CURRENT_TIMESTAMP`)
- `updated_at` (timestamp default `CURRENT_TIMESTAMP`)

Catatan: sintaks migrasi sudah disesuaikan untuk **PostgreSQL**.

## Arsitektur Singkat

- `src/main.rs`
	- inisialisasi `.env`
	- koneksi DB (`config::database::connect`)
	- setup `Router`
	- start server Axum

- `src/config/database.rs`
	- membuat `PgPool` dari `DATABASE_URL`

- `src/utils/jwt.rs`
	- generate dan verifikasi token JWT (`Claims { sub, exp }`)

- `src/utils/response.rs`
	- format response API generik (`ApiResponse<T>`)

## Status Implementasi Endpoint

Beberapa modul endpoint/auth sudah disiapkan, namun routing endpoint publik/protected masih perlu dirangkai penuh di router utama.

Dengan kata lain, proyek sudah siap sebagai **fondasi backend auth**, tetapi masih perlu penyelesaian wiring handler + route agar endpoint bisa diakses end-to-end.

## Troubleshooting

### 1) Gagal migrasi dengan error dekat `ON`

Penyebab umum: memakai sintaks MySQL (`ON UPDATE CURRENT_TIMESTAMP`) di PostgreSQL.

Solusi: gunakan sintaks PostgreSQL seperti di migrasi saat ini.

### 2) Error build terkait `aws-lc-sys` / NASM di Windows

Penyebab: backend crypto native `aws_lc_rs` membutuhkan toolchain tambahan (NASM, dsb).

Solusi yang dipakai di proyek ini: gunakan fitur `jsonwebtoken` backend `rust_crypto` agar build lebih sederhana di Windows.

### 3) Server tidak bisa konek database

Periksa:

- PostgreSQL sedang berjalan
- `DATABASE_URL` benar
- Database target sudah dibuat

## Pengembangan Lanjutan (Saran)

- Tambah route auth (`/api/auth/register`, `/api/auth/login`)
- Rapikan konsistensi layer DB/query (PostgreSQL secara penuh)
- Tambah middleware auth ke route protected
- Tambah testing (unit/integration)
- Tambah dokumentasi API (OpenAPI/Swagger)

## Lisensi

Belum ditentukan. Tambahkan file lisensi sesuai kebutuhan proyek.
