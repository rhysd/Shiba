### How to create `icon.ico`

Install [imagemagick](https://imagemagick.org/index.php) and run the following command:

```sh
convert shibainu.png -define icon:auto-resize=256,128,48,32,16 icon.ico
```

### How to create `icon.icns`

On macOS, run the following command:

```sh
iconutil -c icns icon.iconset
```
