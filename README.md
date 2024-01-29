# silly-png

embed shellscripts and files into png files!

[![crates.io](https://img.shields.io/crates/v/silly-png.svg)](https://crates.io/crates/silly-png)

see [silly.png](silly.png) for an example

![silly.png](silly.png)

## usage

```sh
# install silly-png
cargo install silly-png

# rickroll
silly-png my_funny_cat_image_real.png scripts/epic_rickroll_script.sh rickroll.mp4
# you can now send my_funny_cat_image_real.silly.png to your friends!

# you can even boot a VM
silly-png amogus.png scripts/qemu.sh AmogOS-v0.2.1.iso
# see qemu.sh for details
```

## example scripts

there are example scripts under the [scripts](scripts) folder go there

## warning

i wrote this between 2:30 and 6:00

## please do not use this to spread malware i will bite your head off

it is also cc0, see [LICENSE](LICENSE) for details
