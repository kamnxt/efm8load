# efm8load
A tiny utility for communicating with the HID bootloader on EFM8 microcontrollers from Silabs.

## Usage
Run `efm8load <filename>`. The file needs to contain valid EFM8 bootloader
commands, each starting with $. To create this file, you can currently use
`hex2boot.exe` from Silabs, which seems to work fine through Wine.
