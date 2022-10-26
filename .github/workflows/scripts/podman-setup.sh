#!/bin/bash
systemctl enable --user podman.socket
systemctl start --user podman.socket
export DOCKER_HOST="unix://$(podman info -f "{{.Host.RemoteSocket.Path}}")"
sudo chmod 666 /var/run/docker.sock
