// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.fml;

public enum LogicalSide {
    CLIENT,
    SERVER;

    public boolean isServer() {
        return this == SERVER;
    }

    public boolean isClient() {
        return this == CLIENT;
    }
}
