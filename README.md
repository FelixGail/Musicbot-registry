# Musicbot-Registry

Keeps a registry of active [Musicbot](https://github.com/BjoernPetersen/MusicBot)s on your public ip.
Intended to be used together with the [Musicbot-web](https://github.com/FelixGail/Musicbot-web) project.

## API

Check [openapi.yaml](openapi.yaml) for the api specification of the MusicBot-registry.

## Usage

Send a POST request for every instance of a bot you are using. Refresh the registration regularly, a registration is
valid for 5 minutes.

To find a bot instance send a simple GET request. The service will match your public IP and check if there are any
musicbot instances registered for it.