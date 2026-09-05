# Aqua Core Icon Production Plan

This document is the source-of-truth handoff list for project-owner-produced
Aqua SVG artwork. Aqua does not derive, trace, recolor, or redistribute icons
from elementary, GNOME, KDE, Apple, or another desktop product.

## Delivery Contract

- Deliver one standalone SVG for each filename in the tables below.
- Use a `0 0 64 64` view box and transparent canvas.
- Keep important geometry inside a 4-unit safe area. Optical overshoot may use
  at most 2 additional units when required by circles or diagonals.
- Do not embed text, raster images, external files, scripts, animation,
  filters, blur, glow, gradients, or product mockup backgrounds.
- Use lowercase kebab-case filenames exactly as listed.
- Symbolic icons use `currentColor` and must remain readable as one color.
- Full-color application icons may use Aqua palette fills, but must also remain
  identifiable in a 24-pixel silhouette check.
- Preserve alpha. Do not add a tile unless the tile is part of the application
  icon itself.
- Light and Dark use the same SVG geometry.
  Theme and interaction colors are renderer tokens, not duplicate files.
- Hover, focus, pressed, selected, disabled, and attention normally reuse the
  same SVG. Produce a separate file only when the meaning or silhouette
  changes, such as `trash-empty.svg` and `trash-full.svg`.
- Every submission must be project-authored and may not contain traced path
  data from third-party icon sets.

## Visual Grammar

Symbolic controls use a consistent 2.5-unit visual stroke, round caps, round
joins, and simple closed silhouettes. Filled regions should be preferred when
a thin stroke would collapse at 16 pixels. Align primary vertical and
horizontal edges to whole view-box units. Avoid detail that disappears at 16,
20, or 24 pixels.

Application icons may be more illustrative, but they still use clear frontal
silhouettes, restrained flat color layers, one dominant object, and no
photographic texture. Aqua identity comes from geometry and color discipline,
not shine or decorative effects.

## Existing Core Set

These SVGs already exist. They remain in scope for style review; replace a file
only when the new drawing is clearly identified as an owner-produced revision.

| File | Role | Kind | Status |
| --- | --- | --- | --- |
| `aqua-drive.svg` | Aqua cloud/storage location | Full color | Existing |
| `battery.svg` | Generic battery status | Symbolic | Existing |
| `browser.svg` | Web browser application | Full color | Existing |
| `files.svg` | Files application | Full color | Existing |
| `home.svg` | Home location | Symbolic | Existing |
| `notification.svg` | Notification center | Symbolic | Existing |
| `settings.svg` | Settings application | Full color | Existing |
| `software.svg` | Software application | Full color | Existing |
| `terminal.svg` | Terminal application | Full color | Existing |
| `trash.svg` | Generic Trash compatibility icon | Full color | Existing |
| `updates.svg` | Updates application | Full color | Existing |
| `volume.svg` | Generic audio status | Symbolic | Existing |
| `wifi.svg` | Generic wireless status | Symbolic | Existing |

## Delivery 1: Shell And Window Essentials

These are the first requested SVGs. They replace the most visible procedural
glyphs in the desktop, launcher, top bar, window chrome, session menu, and
shared controls.

| File | Role | Kind |
| --- | --- | --- |
| `applications.svg` | Open Applications | Symbolic |
| `search.svg` | Global search | Symbolic |
| `workspace.svg` | Workspace identity/selection | Symbolic |
| `window-minimize.svg` | Minimize window | Symbolic |
| `window-maximize.svg` | Maximize window | Symbolic |
| `window-restore.svg` | Restore maximized window | Symbolic |
| `window-close.svg` | Close window | Symbolic |
| `menu.svg` | Open primary menu | Symbolic |
| `more-horizontal.svg` | More actions | Symbolic |
| `chevron-down.svg` | Expand menu/select | Symbolic |
| `chevron-right.svg` | Reveal/navigate forward | Symbolic |
| `arrow-back.svg` | Back navigation | Symbolic |
| `arrow-forward.svg` | Forward navigation | Symbolic |
| `arrow-up.svg` | Parent/up navigation | Symbolic |
| `add.svg` | Add/create | Symbolic |
| `remove.svg` | Remove | Symbolic |
| `edit.svg` | Edit/rename | Symbolic |
| `save.svg` | Save | Symbolic |
| `refresh.svg` | Refresh/reload | Symbolic |
| `undo.svg` | Undo | Symbolic |
| `redo.svg` | Redo | Symbolic |
| `check.svg` | Success/confirmed | Symbolic |
| `information.svg` | Informational status | Symbolic |
| `warning.svg` | Warning | Symbolic |
| `error.svg` | Error/failure | Symbolic |
| `help.svg` | Help | Symbolic |
| `view-grid.svg` | Grid view | Symbolic |
| `view-list.svg` | List view | Symbolic |
| `filter.svg` | Filter | Symbolic |
| `sort.svg` | Sort | Symbolic |
| `lock.svg` | Lock session/security | Symbolic |
| `sleep.svg` | Suspend/sleep | Symbolic |
| `restart.svg` | Restart session/system | Symbolic |
| `power-off.svg` | Shut down | Symbolic |
| `user.svg` | User/account | Symbolic |
| `keyboard.svg` | Keyboard layout/input | Symbolic |
| `network-wired.svg` | Wired network status | Symbolic |
| `bluetooth.svg` | Bluetooth status | Symbolic |

## Delivery 2: Status Variants

Status variants change meaning or silhouette and therefore require separate
files. Numeric suffixes describe semantic level, not a percentage rendered as
text.

| File | Role | Kind |
| --- | --- | --- |
| `battery-0.svg` | Empty battery | Symbolic |
| `battery-25.svg` | Low battery | Symbolic |
| `battery-50.svg` | Half battery | Symbolic |
| `battery-75.svg` | High battery | Symbolic |
| `battery-100.svg` | Full battery | Symbolic |
| `battery-charging.svg` | Charging battery | Symbolic |
| `wifi-off.svg` | Wireless disabled/unavailable | Symbolic |
| `wifi-low.svg` | Weak wireless signal | Symbolic |
| `wifi-medium.svg` | Medium wireless signal | Symbolic |
| `wifi-high.svg` | Strong wireless signal | Symbolic |
| `volume-muted.svg` | Muted audio | Symbolic |
| `volume-low.svg` | Low audio level | Symbolic |
| `volume-medium.svg` | Medium audio level | Symbolic |
| `volume-high.svg` | High audio level | Symbolic |
| `notification-unread.svg` | Unread notification attention | Symbolic |

## Delivery 3: Files And Places

These cover the Files sidebar, grid/list rows, Trash, file chooser, properties,
installer storage selection, and global search results.

| File | Role | Kind |
| --- | --- | --- |
| `folder.svg` | Generic folder | Full color |
| `folder-open.svg` | Open/current folder | Full color |
| `recent.svg` | Recent files | Symbolic |
| `starred.svg` | Starred/favorite files | Symbolic |
| `desktop.svg` | Desktop location | Symbolic |
| `documents.svg` | Documents location | Symbolic |
| `downloads.svg` | Downloads location | Symbolic |
| `pictures.svg` | Pictures location | Symbolic |
| `music.svg` | Music location | Symbolic |
| `videos.svg` | Videos location | Symbolic |
| `computer.svg` | Local computer | Symbolic |
| `drive-harddisk.svg` | Internal storage device | Full color |
| `drive-removable.svg` | Removable storage device | Full color |
| `drive-optical.svg` | Optical media | Full color |
| `network-location.svg` | Network location | Symbolic |
| `trash-empty.svg` | Empty Trash | Full color |
| `trash-full.svg` | Trash containing items | Full color |
| `file-generic.svg` | Unknown/generic file | Full color |
| `file-text.svg` | Plain text/document file | Full color |
| `file-image.svg` | Image file | Full color |
| `file-audio.svg` | Audio file | Full color |
| `file-video.svg` | Video file | Full color |
| `file-pdf.svg` | PDF document | Full color |
| `file-archive.svg` | Archive file | Full color |
| `file-code.svg` | Source code/script | Full color |
| `file-executable.svg` | Executable/application file | Full color |
| `link.svg` | Symbolic link/shortcut | Symbolic |
| `properties.svg` | Item properties | Symbolic |

## Delivery 4: Settings Categories

All category icons are symbolic. The selected state is token-colored by the
renderer and does not need another SVG.

| File | Role |
| --- | --- |
| `system.svg` | System overview |
| `appearance.svg` | Appearance/theme |
| `display.svg` | Displays |
| `desktop-settings.svg` | Desktop behavior |
| `applications-settings.svg` | Application defaults/permissions |
| `notifications-settings.svg` | Notification settings |
| `search-settings.svg` | Search providers/indexing |
| `multitasking.svg` | Workspaces and multitasking |
| `network-settings.svg` | Network settings |
| `bluetooth-settings.svg` | Bluetooth settings |
| `sound-settings.svg` | Sound settings |
| `power-settings.svg` | Power and battery settings |
| `users-settings.svg` | Users and accounts |
| `date-time.svg` | Date and time |
| `region-language.svg` | Region and language |
| `privacy-security.svg` | Privacy and security |
| `accessibility.svg` | Accessibility |
| `mouse-touchpad.svg` | Mouse and touchpad |
| `storage.svg` | Storage usage |
| `backup.svg` | Backup |
| `developer.svg` | Developer options |
| `system-log.svg` | System log |
| `about.svg` | About Aqua Linux |

## Delivery 5: First-Party Application Expansion

These are full-color application icons. They are requested only for
applications or visual primitives already named in the v1 contracts; this is
not a promise that every application ships in the first image.

| File | Application |
| --- | --- |
| `installer.svg` | Aqua Linux Installer |
| `calendar.svg` | Calendar |
| `photos.svg` | Photos |
| `music-app.svg` | Music application |
| `camera.svg` | Camera |
| `text-editor.svg` | Text Editor |
| `calculator.svg` | Calculator |
| `system-monitor.svg` | System Monitor |
| `archive-manager.svg` | Archive Manager |
| `screenshot.svg` | Screenshot tool |
| `notes.svg` | Notes |

## Delivery And Review

Submit Delivery 1 first. A useful first review batch is:

1. `applications.svg`
2. `search.svg`
3. `window-minimize.svg`
4. `window-maximize.svg`
5. `window-restore.svg`
6. `window-close.svg`
7. `arrow-back.svg`
8. `arrow-forward.svg`
9. `chevron-down.svg`
10. `more-horizontal.svg`
11. `check.svg`
12. `warning.svg`
13. `lock.svg`
14. `sleep.svg`
15. `restart.svg`
16. `power-off.svg`

Place delivered files in `docs/aqua-linux/assets/icons/aqua/`. Each review
checks XML safety, license/provenance, filename, view box, forbidden SVG
features, transparent rendering, 16/20/24/32/48/64/128 pixel output, all four
themes, and silhouette stability. Passing SVGs are then added to the runtime
manifest and packaged image; merely listing a filename here does not make it a
runtime asset.

The existing 13-icon core set now passes the renderer's bounded static-SVG
loader and the committed scale-native fixture matrix. This acceptance does not
approve any undelivered filename from Delivery 1 through Delivery 5. New files
must still complete the same source, license, silhouette, theme, state, size,
and output-scale review before entering the runtime manifest.
