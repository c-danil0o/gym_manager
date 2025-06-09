# GymMan - Desktop Gym Management System

<!-- ![App Screenshot](screenshots/app_overview.png) -->

Aka Gym is a comprehensive desktop application designed to simplify and streamline the daily operations of a fitness center. It empowers gym owners and staff to manage members, track attendance, oversee memberships, and gain valuable insights into their gym's performance, all from a secure, user-friendly interface that works even offline.

## ⭐ Key Functionalities

*   **Effortless Member Management:**
    *   Quickly add new members with their essential details and assign unique card IDs.
    *   Easily search, view, and update existing member profiles.
    *   Maintain a clear record of each member's journey with the gym.

*   **Flexible Membership Control:**
    *   Define and customize various membership types (e.g., monthly, annual, class packs, student discounts).
    *   Assign memberships to members with specific start and end dates.
    *   Track the status of each membership (active, pending, expired, suspended, cancelled).
    *   Manage visit-based passes by automatically decrementing remaining visits upon entry.

*   **Streamlined Gym Entry & Access Control:**
    *   **Fast Card Scanning:** A dedicated scanner interface for quick member check-in using card IDs (supports keyboard-emulating scanners).
    *   **Instant Entry Validation:** Automatically checks membership validity, status, visit limits, and daily entry rules.
    *   **Clear Visual Feedback:** Immediate on-screen dialog indicating "Access Granted" or "Access Denied" with clear reasons.
    *   **Time-Based Access:** Supports "enter by" rules, allowing memberships to grant access only before a certain time of day.
    *   **Daily Entry Prevention:** Prevents multiple check-ins for the same member on the same day if configured.
    *   **One-Time Entry Support:** Log even single entries without requiring a membership, useful for one-time visitors or trials.

*   **Comprehensive Entry Logging:**
    *   Automatically records every scan attempt, whether access is granted or denied, along with timestamps and reasons.
    *   View a list of the most recent gym entries directly on the scanner page for quick monitoring.
    *   Access a detailed entry log for historical review and attendance tracking.

*   **Insightful Analytics Dashboard:** *(Visualize your gym's pulse)*
    *   **Membership Popularity:** See which membership types are the most popular among your members.
    *   **Growth Trends:** Track the number of active memberships over time to monitor business health.
    *   **Peak Hours:** Identify the busiest times of day and days of the week to optimize staffing and class schedules.
    *   **Revenue Insights:** Understand revenue generated per membership type (based on purchases).
    *   *(More analytics can be added based on needs)*

*   **Reliable Data Management:**
    *   **Offline Operation:** Core functionalities like member check-in, member lookup, and data entry work seamlessly without an internet connection.
    *   **Secure Local Storage:** All data is stored locally on the computer in local database.
    *   **Remote Data Backup:** Option to configure periodic backups of the local database to a remote server via a secure API, ensuring data safety.
    *   **Data Restore:** Ability to restore data from a remote backup if needed.

*   **User-Friendly Experience:**
    *   **Intuitive Interface:** Clean and modern design for ease of use.
    *   **Multi-Language Support:** Interface available in multiple languages.
    *   **Customizable Theme:** Choose between light, dark, or system default appearance.
    *   **Personalized Settings:** Configure timezone for display, backup preferences, and other application settings.
    *   **Secure Admin Access:** Application access is protected by an administrator login.
    *   **Separate Worker Account :** Worker accounts can be created with limited access to specific functionalities, such as member management and entry logging.


## 🎯 Who is this for?

*   Small to medium-sized gym owners.
*   Fitness studio managers.
*   Anyone needing a reliable, offline-first system to manage memberships and facility access.

## 🚀 Getting Started


1.  **Installation:** Download the latest release for your operating system from the [Releases Page](https://github.com/c-danil0o/gym_manager/releases).
    *   **Windows:** Download the `.msi` installer.
    *   **Linux:** Download the appropriate package for your distribution (e.g., `.deb`, `.rpm`, or AppImage).
2.  **First Run:**
    *   On the first launch, a local database will be created.
    *   A default administrator account will be set up:
        *   **Username:** `admin`
        *   **Password:** `admin`
    *   **IMPORTANT:** Please change the default admin password immediately via the application settings for security.
3.  **Configuration (Optional but Recommended):**
    *   Navigate to the "Settings" section within the application.
    *   Set your preferred language, theme, and display timezone.
    *   To enable remote data backups, provide your secure "Backup API URL" (obtained from your separate API setup). Configure the backup frequency.

<!-- ## 🖼️ Application Preview


**Scanner Interface & Entry Status:**
![Scanner Page](screenshots/scanner_page_success.png)

**Member Overview:**
![Member Overview](screenshots/member_overview.png)

**Membership Management:**
![Membership Management](screenshots/membership_management_ui.png)

**Analytics Dashboard:**
![Analytics](screenshots/analytics_preview.png) -->


## 🛠️ For Developers (Technical Details & Contribution)

<details>
<summary>Click to expand technical details and contribution guide</summary>

### Tech Stack
*   **Package manager:** Bun
*   **Desktop Framework:** Tauri (v2)
*   **Frontend:** SvelteKit, TypeScript, shadcn-svelte, Tailwind CSS, ParaglideJS
*   **Backend (Tauri Core):** Rust, Tokio, SQLx, SQLite
*   *(For a more detailed list, see the original tech stack section or `Cargo.toml` / `package.json`)*


### Project Structure
*   `src/`: SvelteKit frontend (routes, components, stores, i18n messages).
*   `src-tauri/`: Rust backend (Tauri setup, commands, database logic, migrations).
*   `messages/`: ParaglideJS translation files.

### Contributing
Contributions are welcome! Please:
1.  Open an issue to discuss significant changes.
2.  Fork the repository.
3.  Create a feature branch.
4.  Make your changes.
5.  Submit a pull request with a clear description of your changes.

</details>

## 📄 License
Distributed under the MIT License. See `LICENSE` file for more information.
