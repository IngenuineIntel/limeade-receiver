#!/usr/bin/sh

cargo build --release

sudo rm /usr/local/bin/limeade-receiver /usr/local/sbin/limeade-receiver

sudo cp ./target/release/limeade-receiver /usr/local/bin/limeade-receiver
sudo ln -s /usr/local/bin/limeade-receiver /usr/local/sbin/limeade-receiver

if ! sudo test -e /etc/limeade/ssh/id_ed25519; then
	sudo ssh-keygen -q -t ed25519 -N "" -f /etc/limeade/ssh/id_ed25519
fi
sudo chown root:limeade /etc/limeade/ssh/id_ed25519 /etc/limeade/ssh/id_ed25519.pub
sudo chmod 0640 /etc/limeade/ssh/id_ed25519 /etc/limeade/ssh/id_ed25519.pub
sudo install -d -o root -g limeade -m 2770 /etc/limeade/history
sudo chown root:limeade /etc/limeade /etc/limeade/history
sudo chmod 2750 /etc/limeade
sudo chmod 2770 /etc/limeade/history
sudo find /etc/limeade/history -type f -exec chown root:limeade {} + -exec chmod 0660 {} +

echo "DONE"
