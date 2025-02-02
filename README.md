# <img src="https://cdn.discordapp.com/attachments/537415745198489633/1094617120538644622/icon.png" width="30"/> LOA Logs

[![GitHub](https://img.shields.io/github/downloads/snoww/loa-logs/total?style=for-the-badge&color=%23ff9800)](https://github.com/snoww/loa-logs/releases/latest)


[![GitHub](https://img.shields.io/github/v/release/snoww/loa-logs?style=flat-square)](https://github.com/snoww/loa-logs/releases)
[![GitHub](https://img.shields.io/github/license/snoww/loa-logs?style=flat-square)](https://github.com/snoww/loa-logs/blob/master/LICENSE)

LOA Logs is a "blazingly fast" open source Lost Ark DPS meter, written in Rust by [Snow](https://github.com/snoww). 

This project is an opinionated flavor of [LOA Details](https://github.com/lost-ark-dev/loa-details) by Herysia and Mathi, but should share very similar user interfaces and settings. The packet sniffing and processing has been completely ported over to Rust, with [`meter-core-rs`](https://github.com/snoww/meter-core-rs). The Rust port could not be made without Herysia and Henjuro's work on [`meter-core`](https://github.com/lost-ark-dev/meter-core). If you wish to support their development, you can do so by supporting their [Patreon](https://patreon.com/Herysia).

This project was designed specifically with hell-raiding in mind.

# Download
https://github.com/snoww/loa-logs/releases

*currently only Windows 7 and up is supported

# Prerequisites
Npcap is required to run LOA Logs.

Download [here](https://npcap.com/#download).

# FAQ
**Q: Meter isn't detecting anything...**

A: There can be multiple reasons. 1. Did you install Npcap? 2. Are you using a traditional VPN (e.g. NordVPN)? You need to disable auto-interface, and select the network interface for your VPN (should be named similar to your vpn name). If that doesn't work, enable raw socket mode. You must restart the meter as admin. 3. Are you using ExitLag? ExitLag should work on auto-interface, since its not really a VPN. However, if its not working on auto interface, you need to enable raw socket mode. You must restart the meter as admin.

**Q: Should I run it in a VM?**

A: I do not run it in a VM with full 10 gems equipped on my character. There is always a risk of getting banned, even in a VM. You can run it in a VM if you want, the meter should work the same.

**Q: Missing `packet.dll`**

A: You need install Npcap. If you already have Npcap installed and error still shows, please uninstall it, and then reinstall the latest version using the link above.

**Q: The installer crashes or takes forever to install**

A: Are you trying to install on a custom install folder with different permissions? You might need to run the installer in administrator mode due to permission issues.

**Q: The meter crashes immediately when trying to open it.**

A: There could be two possible reasons. 1. The meter needs Microsoft Edge Webview2 Runtime to run. Yours is probably missing or out of date. Go uninstall it first (it won't let you install it if you have an older version installed), then download and install from [here](https://go.microsoft.com/fwlink/p/?LinkId=2124703) (https://go.microsoft.com/fwlink/p/?LinkId=2124703). 2. If you installed the meter in another folder that might require elevated permissions, you would need to run the program in administrator mode.

**Q: The meter window lags a lot when dragging around.**

A: Are you on Windows 11? Disable blur in the settings. If you wish to have a dark background with blur disabled, also disable the transparency setting to have a pseudo dark mode.

**Q: Why isn't my item level shown next to my name when others have it?**

A: You opened the meter too late, and it wasn't able to get your character information. It is doing its best by guessing. You can fix this by: switching characters, or changing parties around. (note: you need to enable "show gear score" in settings to show item level)

**Q: There are too many/too few columns in the meter.**

A: You can change whatever column you want to show in the settings. TIP: you can `SHIFT+SCROLL` to scroll horizontally.

**Q: Are you going to implement rDPS like LOA Details?**

A: No. If you wish to see rDPS, please use [LOA Details](https://github.com/lost-ark-dev/loa-details). They have spent a lot of effort simulating stats and buffs to make it work, and I am way too lazy to port that here. You can have both tools running at the same time if you wish.

**Q: Help, my issue isn't listed here.**

A: Create an issue here on GitHub, or send me a DM on Discord.

**Q: Is it really "blazingly fast"?**

A: [Yes.](https://cdn.discordapp.com/attachments/537415745198489633/1134417704732872704/t7ns9qtb5gh81.png)



## Screenshots
### In-game Overlay (optional Boss HP bar)
![log_image](https://cdn.discordapp.com/attachments/537415745198489633/1100293328995614750/image.png)

### Past Encounters
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1100229380652929044/image.png" width="500"/>

### Damage Breakdown with DPS Charts
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1100220743846989935/image.png" width="500"/>

### Skill Breakdown
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1100230099640524861/image.png" width="500"/>

### Buff Uptime Tracking
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1100220998378324068/image.png" width="500"/>

### Identity Tracking (only for yourself)
#### Arcana Card Tracking
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1100220506231287818/image.png" width="500"/>

#### Bard/Artist Bubble Tracking
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1100239509754490931/image.png" width="500"/>

### Stagger Tracking (only bosses with visible stagger bar)
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1100300320392871986/image.png" width="500"/>

### Skill Cast Log
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1095046175171813436/image.png" width="500"/>
