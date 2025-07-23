{
  packages
}: (
  {
    lib,
    pkgs,
    config,
    ...
  }:

  let
    inherit (lib)
      mkEnableOption
      mkIf
      mkOption
      optionalAttrs
      optional
      mkPackageOption;
    inherit (lib.types)
      bool
      path
      str
      submodule
      number
      array
      listOf;

    cfg = config.services.mental-instability-bot;

    format = pkgs.formats.toml { };
    configFile = format.generate "mental-instability-bot-config" cfg.config;
    configDir = pkgs.writeTextDir "config.toml" (builtins.readFile configFile);
  in
  {
    options.services.mental-instability-bot = {
      enable = mkEnableOption "Mental Instability Bot";

      package = mkPackageOption packages.${pkgs.stdenv.hostPlatform.system} "default" { };

      config = mkOption {
        type = submodule {
          freeformType = format.type;

          options = {
            token = mkOption {
              type = str;
              description = ''
                The bot token.
              '';
            };

            log_extensions = mkOption {
              type = listOf str;
              description = ''
                Extensions of log files.
              '';
              default = [
                  ".log"
                  "-client.txt"
                  "-server.txt"
                  "message.txt"
                  ".log.gz"
              ];
            };
          };
        };

        default = { };

        description = ''
          Values for the config file
        '';
      };

      user = mkOption {
        type = str;
        default = "mental-instability-bot";
        description = "User account under which the bot runs.";
      };

      group = mkOption {
        type = str;
        default = "mental-instability-bot";
        description = "Group account under which the bot runs.";
      };
    };

    config = mkIf cfg.enable {
      systemd.services = {
        mental-instability-bot = {
          description = "Mental Instability Bot";
          after = [ "network.target" ];
          wantedBy = [ "multi-user.target" ];
          restartTriggers = [
            cfg.package
            configFile
          ];

          serviceConfig = {
            Type = "simple";
            User = cfg.user;
            Group = cfg.group;
            WorkingDirectory = cfg.configDir;
            ExecStart = "${cfg.package}/bin/mental-instability-bot";
            Restart = "always";
          };
        };
      };

      users.users = optionalAttrs (cfg.user == "mental-instability-bot") {
        mental-instability-bot = {
          isSystemUser = true;
          group = cfg.group;
        };
      };

      users.groups = optionalAttrs (cfg.group == "mental-instability-bot") {
        mental-instability-bot = { };
      };
    };
  }
)