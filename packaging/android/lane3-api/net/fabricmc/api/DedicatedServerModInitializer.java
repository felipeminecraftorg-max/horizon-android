// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.fabricmc.api;

@FunctionalInterface
public interface DedicatedServerModInitializer {
    void onInitializeServer();
}
