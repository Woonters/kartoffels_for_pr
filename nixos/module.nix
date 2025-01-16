{ self }:
{
  config,
  lib,
  pkgs,
  ...
}:
with lib;

let
  cfg = config.services.kartoffels;

in
{
  options = {
    services.kartoffels = {
      enable = mkEnableOption "kartoffels";

      app = {
        package = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.app;
        };

        data = mkOption {
          type = types.str;
          default = "/var/lib/kartoffels";
        };

        http = mkOption {
          type = types.nullOr types.str;
          default = "0.0.0.0:81";
        };

        ssh = mkOption {
          type = types.nullOr types.str;
          default = "0.0.0.0:22";
        };

        debug = mkOption {
          type = types.bool;
          default = false;
        };
      };

      web = {
        package = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.web;
        };
      };

      nginx = {
        enable = mkEnableOption "nginx proxy";

        package = mkOption {
          type = types.package;
          default = pkgs.nginx;
        };

        listen = {
          addr = mkOption {
            type = types.str;
            default = "0.0.0.0";
          };

          port = mkOption {
            type = types.int;
            default = 80;
          };
        };
      };
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [
      cfg.app.package
    ];

    services.nginx = mkIf cfg.nginx.enable {
      enable = true;

      virtualHosts = {
        default = {
          default = true;

          listen = [
            {
              addr = cfg.nginx.listen.addr;
              port = cfg.nginx.listen.port;
            }
          ];

          locations = {
            "/" = {
              root = "${cfg.web.package}";
            };

            "/api" = {
              proxyPass = "http://${cfg.app.http}/";
              proxyWebsockets = true;
            };

            "/api/" = {
              proxyPass = "http://${cfg.app.http}/";
              proxyWebsockets = true;
            };
          };
        };
      };
    };

    systemd.services.kartoffels = {
      script = ''
        mkdir -p "${cfg.app.data}"

        ${cfg.app.package}/bin/kartoffels serve \
            '${cfg.app.data}' \
            ${optionalString (cfg.app.http != null) "--http ${cfg.app.http}"} \
            ${optionalString (cfg.app.ssh != null) "--ssh ${cfg.app.ssh}"} \
            ${optionalString cfg.app.debug "--debug"}
      '';

      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
    };
  };
}
