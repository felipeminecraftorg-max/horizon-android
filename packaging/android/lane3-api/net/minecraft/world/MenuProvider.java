// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world;

/** A menu with its title. */
public interface MenuProvider extends MenuConstructor {
    net.minecraft.network.chat.Component getDisplayName();

    default boolean shouldTriggerClientSideContainerClosingOnOpen() {
        return true;
    }
}
