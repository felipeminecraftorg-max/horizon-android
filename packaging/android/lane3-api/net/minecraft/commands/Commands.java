// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.commands;

import dev.horizon.android.lane3.Native;

/** The server's commands: Horizon runs a mod's as the acting player, or at the source's place. */
public class Commands {
    public static final Commands CURRENT = new Commands();

    public void performPrefixedCommand(CommandSourceStack source, String command) {
        String line = command.startsWith("/") ? command.substring(1) : command;
        if (source != null && source.entity == null) {
            // A command of the world's (no entity): it runs where the source is.
            line = "execute positioned " + source.position.x + " " + source.position.y + " " + source.position.z + " run " + line;
        }
        Native.command(line);
    }

    public int performCommand(Object parse, String command) {
        performPrefixedCommand(null, command);
        return 1;
    }
}
