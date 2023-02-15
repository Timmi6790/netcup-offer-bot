<br/>
<p align="center">
  <h3 align="center">Netcup Offer Bot</h3>

  <p align="center">
    <a href="https://github.com/Timmi6790/netcup-offer-bot/issues">Report Bug</a>
    .
    <a href="https://github.com/Timmi6790/netcup-offer-bot/issues">Request Feature</a>
  </p>
</p>

![Docker Image Version (latest semver)](https://img.shields.io/docker/v/timmi6790/netcup-offer-bot)![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/Timmi6790/netcup-offer-bot/clippy.yml) ![Issues](https://img.shields.io/github/issues/Timmi6790/netcup-offer-bot) ![License](https://img.shields.io/github/license/Timmi6790/netcup-offer-bot)

## About The Project

RSS feed listener to discord webhook for https://www.netcup-sonderangebote.de/

### Installation - Helm chart

1. Clone the helm chart from the repo

2. Overwrite values

```yaml
image:
  repository: timmi6790/netcup-offer-bot
  tag: latest
  pullPolicy: IfNotPresent

env:
  # Optional sentry url
  sentryDns: ""
  # Discord webhook url
  webHook: ""
  # Check interval in seconds
  checkInterval: "180"

persistence:
  data:
    accessMode: ReadWriteOnce
    size: 10Mi

resources:
  limits:
    memory: 10Mi
  requests:
    memory: 5Mi
```

## License

Distributed under the MIT License. See [LICENSE](https://github.com/Timmi6790/netcup-offer-bot/blob/main/LICENSE.md) for
more information.