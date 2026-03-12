#!/bin/bash
set -e

echo "🚀 Instalando rspot..."

cargo build --release

sudo cp target/release/rspot /usr/local/bin/rspot
echo "✅ Binario instalado"

mkdir -p ~/.config/autostart
cat > ~/.config/autostart/rspot.desktop << EOF
[Desktop Entry]
Type=Application
Name=rspot
Exec=bash -c '/usr/local/bin/rspot'
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
EOF
echo "✅ Autostart configurado"

if [ ! -f ~/.config/rspot/config.toml ]; then
mkdir -p ~/.config/rspot
cat > ~/.config/rspot/config.toml << EOF
[window]
width = 500
height = 350
max_height = 580
[colors]
background = "#2b2b2b"
opacity = 0.9
selected_item_color = "#5294e2"

[font]
font_size = 14
font_color = "#ffffff"

EOF
echo "✅ Config creado en ~/.config/rspot/config.toml"
else
    echo "⚠️  Config ya existe, no se sobreescribe"
fi

gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings "['/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/rspot/']"

gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/rspot/ name "rspot"

gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/rspot/ command "gdbus call --session --dest com.davidgl.Rspot --object-path /com/davidgl/Rspot --method com.davidgl.Rspot.Show"

gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/rspot/ binding "<Super>r"

echo "✅ Hotkey Super+Space configurado"

pkill rspot 2>/dev/null || true  
WGPU_BACKEND=gl nohup /usr/local/bin/rspot &>/dev/null &

echo "🎉 Instalación completa"
