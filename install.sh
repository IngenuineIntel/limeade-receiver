#!/usr/bin/sh


cargo build --release
sudo install -D -m 0755 ./target/release/limeade-receiver /usr/sbin/limeade-receiver
sudo install -d -o root -g limeade -m 0750 /etc/limeade/ssh
if ! sudo test -e /etc/limeade/ssh/id_ed25519; then
	sudo ssh-keygen -q -t ed25519 -N "" -f /etc/limeade/ssh/id_ed25519
fi
sudo chown root:limeade /etc/limeade/ssh/id_ed25519 /etc/limeade/ssh/id_ed25519.pub
sudo chmod 0640 /etc/limeade/ssh/id_ed25519 /etc/limeade/ssh/id_ed25519.pub
sudo install -d -o root -g limeade -m 2770 /etc/limeade/history
sudo chown root:limeade /etc/limeade /etc/limeade/history
sudo chmod 2750 /etc/limeade
sudo chmod 2770 /etc/limeade/history
