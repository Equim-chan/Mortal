import toml
import os

config_file = os.environ.get('MORTAL_CFG', 'config.toml')
with open(config_file) as f:
    config = toml.load(f)
