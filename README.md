# Gym Manager

A desktop app for managing gym members, memberships, and entry access. Works offline.

![scanner](./images/scanner.png)
![member_details](./images/member_details.png)
![analytics](./images/analytics.png)

## Features

- **Members** — add, search, and update member profiles with card IDs
- **Memberships** — define membership types (monthly, visit-based, etc.), assign them to members, and track status (active, pending, expired, suspended)
- **Entry scanning** — check-in via card scanner with instant access validation; enforces visit limits, time-based access rules, and daily entry prevention
- **Entry log** — full history of every scan attempt with timestamps and reasons
- **Analytics** — membership distribution, active membership trends, peak hours, revenue by type
- **Backup** — optional periodic backup to a remote server with restore support
- **Multi-user** — admin and worker accounts with different access levels
- **Multi-language** — UI available in multiple languages

## Getting Started

Download the latest release from the [Releases page](https://github.com/c-danil0o/gym_manager/releases).

- **Windows:** `.msi` installer
- **Linux:** `.deb`, `.rpm`, or AppImage

On first launch, a local database is created and a default admin account is set up:

- Username: `admin`
- Password: `admin`

**Change the default password immediately after first login.**

## Tech Stack

- **Frontend:** SvelteKit, TypeScript, Tailwind CSS, shadcn-svelte, ParaglideJS
- **Backend:** Rust, Tauri v2, SQLx, SQLite
- **Package manager:** Bun

## License

MIT — see `LICENSE` for details.
