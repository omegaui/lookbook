# Desktop screenshot credits

The Desktops page shows one real screenshot per desktop. Linux & You never bundles them: each card loads a 960px thumbnail from Wikimedia Commons when it first appears on screen. Each image's page on Commons holds its full license and author details; the card's tooltip names the author and license.

| Desktop | Image | Author | License |
| --- | --- | --- | --- |
| GNOME | [GNOME_Shell.png](https://commons.wikimedia.org/wiki/File:GNOME_Shell.png) | Guilieb | CC BY-SA 4.0 |
| KDE Plasma | [Kde_Plasma_6.png](https://commons.wikimedia.org/wiki/File:Kde_Plasma_6.png) | Axo1otl | GPL |
| COSMIC | [Pop!_OS_24.04_LTS_default_desktop_-_English.png](https://commons.wikimedia.org/wiki/File:Pop!_OS_24.04_LTS_default_desktop_-_English.png) | System76 | GPL-3.0 |
| Cinnamon | [Linux_Mint_Cinnamon_22.3.png](https://commons.wikimedia.org/wiki/File:Linux_Mint_Cinnamon_22.3.png) | Breno Martins de Oliveira Vasconcelos | GPL-3.0 |
| Xfce | [XFCE_4.20.png](https://commons.wikimedia.org/wiki/File:XFCE_4.20.png) | Unknown author | GPL |
| Budgie | [BudgieDesktop-v10.7.jpg](https://commons.wikimedia.org/wiki/File:BudgieDesktop-v10.7.jpg) | Joshua Strobl | Apache-2.0 |
| MATE | [Mate-desktop-1.26.en.png](https://commons.wikimedia.org/wiki/File:Mate-desktop-1.26.en.png) | the MATE developers | GPL |
| LXQt | [LXQt_2.0.0_Ambiance_screenshot.png](https://commons.wikimedia.org/wiki/File:LXQt_2.0.0_Ambiance_screenshot.png) | LXQt developers | CC BY-SA 3.0 |
| Pantheon | [Elementary_OS_8.0_default_desktop_-_English.png](https://commons.wikimedia.org/wiki/File:Elementary_OS_8.0_default_desktop_-_English.png) | elementary, Inc. | GPL-3.0 |

To change a screenshot, edit the desktop's `screenshot` entry in `data/resources/catalog/catalog.json`. Pick a file under a free license, and use the 960px thumbnail address the Commons API returns (`prop=imageinfo&iiurlwidth=960`): Commons only serves standard thumbnail widths, and other widths answer with an error page.
