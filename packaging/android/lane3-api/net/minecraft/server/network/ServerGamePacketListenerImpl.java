// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.server.network;

import dev.horizon.android.lane3.Native;

/** A player's connection to the server. */
public class ServerGamePacketListenerImpl {
    public void teleport(double x, double y, double z, float yaw, float pitch) {
        Native.command("tp @s " + x + " " + y + " " + z + " " + yaw + " " + pitch);
    }
}
