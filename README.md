# Musicbot-Registry

Keeps a registry of active [Musicbot](https://github.com/BjoernPetersen/MusicBot)s on your public ip.
Intended to be used together with the [Musicbot-web](https://github.com/FelixGail/Musicbot-web) project.

## Endpoints

 -  ### GET "/"

    - **Description:** Returns registered musicbot instances
    - **Responses:** 
    
        -   200:<br>
            Body:
            ```json
            [
                {
                    "name": "name of the bot",
                    "domain": "Domain of the bot, e.g. sample.instance.kiu.party",
                    "port": 41239,
                    "updated": "number; milliseconds since registration/update"
                }
            ]
            ```
 -  ### POST "/"

    - **Description:** Register a new musicbot instance
    - **Request-Body:**

        ```json
        {
            "domain": "Domain of the bot, e.g. sample.instance.kiu.party",
            "port": 41239,
            "name": "Name of the bot"
        }
        ```
    - **Responses:**

        - 201: Success
        - 400: Bad Request
        
## Usage

Send a POST request for every instance of a bot you are using. Refresh the registration regularly, a registration is
valid for 5 minutes.

To find a bot instance send a simple GET request. The service will match your public IP and check if there are any
musicbot instances registered for it.