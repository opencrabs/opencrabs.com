# The crab lives inside the phone

<div class="story-meta">

*X · Twitter* **Android · On-device**

</div>

Daniel rooted a Xiaomi Mi 8, dropped OpenCrabs into an Ubuntu chroot on the device, and pointed a Flutter app at it over ACP. Then the crab wrote an Android app, built the APK on the phone with aapt2, javac, d8 and apksigner, and installed it. No PC, no cloud, no Gradle, no SDK.

<div class="story-author">

Daniel · @iamloonix

</div>

> Some say that Flutter sucks… yeah (ironic) @opencrabs inside the phone linked to a2a on a custom flutter app. It can change the OS. It can code android app, compile and open it… all inside the freakin' phone

> First app fully built inside the phone using @opencrabs yes, code made inside the phone, apk built and open. No need for a pc ;)

**The device:** a Xiaomi Mi 8 (`dipper`), Android 10 / API 29, MIUI 12, aarch64, kernel 4.9.186, 5.6 GB of RAM. Root came from the boot image itself: Magisk patches it on the phone, fastboot flashes it, and a verifier unpacks the result to inspect the ramdisk instead of trusting the patch log. Live now: `magisk -c` reports `30.7:MAGISK:R`.

**Getting a Linux userland onto Android took two attempts.** Generation one shipped the `aarch64-linux-android` binary as a Magisk module with a supervisor script, which meant a 59-file cfg port and a glibc-versus-musl fight. Generation two, the one that runs today, is an Ubuntu 24.04.5 chroot at glibc 2.39, which is what the released `linux-arm64` binary actually wants. The trap that cost the most time: `/apex` is a tmpfs of per-apex bind mounts, so a single bind of `/apex` copies an empty directory and every Android binary dies with "required file not found", which reads like a permission error but is a missing interpreter. The cure is binding each `/apex/*` entry on its own.

**Then the app.** Daniel wanted the conversation UI he likes in the web, built natively in Flutter, not a WebView. So the app speaks ACP (JSON-RPC 2.0 over stdio) to `opencrabs acp` running in the chroot, the same protocol the TUI uses: live reasoning chunks, tool cards, permission prompts, token usage. No browser, no server, no open port.

- **Survives a reboot with nobody touching the phone:** a JobScheduler job that re-arms itself unconditionally, a service hook that waits for `sys.boot_completed` and re-establishes the seven mounts, and a loop that reapplies MIUI's idle and autostart appops because the powerkeeper clears the battery exemption about 20 s after boot
- **Proof, from logcat after a real reboot:** handshake ok at 12 s, service up, battery exemption settled, then a headless self test of **12 pass / 0 fail, GREEN** in 3.7 s covering the binary version, shell, sessions, boot wiring, service state, battery exemption, voice recognizer, mic permission, widget binding, ACP ready and a real prompt
- **The APK built on the phone:** MIUI ships its screen-recorder tile with only a 20 KB stub behind it, so tapping it opens the app store and records nothing. The crab wrote a real Quick Settings tile, built it in six steps with no Gradle and no SDK manager, and put it in the panel next to WiFi. **12,784 bytes.**
- **The build chain:** `aapt2 compile` → `aapt2 link` → `javac --release 8` against `android.jar` → `d8` from R8 → zip the dex in → `zipalign` → `apksigner sign` and `verify`
- **Installed from inside the chroot**, reaching Android's package manager through `nsenter -t 1 -m`, then granted root by writing a Magisk policy row and added to the shade with `cmd statusbar add-tile`
- **One body, two brains:** ACP is stdio and `ssh -T` is also stdio, so the phone can point at a Mac on the LAN with no bridge and no open port. Measured: `initialize` in 1.2 s, remote session list of 40, self test 9/9 GREEN against the remote brain

**The part that matters:** the tile does not record anything itself. It shells out, through Magisk's `su`, to the same recorder binary in the chroot that the `/rec` chat command uses, so there is one recorder, one state file, one output folder. The crab wrote an Android app that is a **handle for a Linux tool inside the same phone**. The app is the thumb, the crab is the brain. That is the whole architecture of this project, in 13 KB.

The reasoning log from that session is the detail worth keeping: the crab stopped to check whether it was really on a phone, and confirmed it from the runtime instead of assuming.

[View source on X ↗](https://x.com/iamloonix/status/2112943101052641605)

[The APK built on the phone ↗](https://x.com/iamloonix/status/2112943720295276787)
