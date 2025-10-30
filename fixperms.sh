#chown -R root:nginx *

find . -type f -exec chmod 644 {} \;

find . -type d -exec chmod 755 {} \;
