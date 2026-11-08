// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.commands;

import net.minecraft.network.chat.Component;

/** Who hears a command's output. */
public interface CommandSource {
    CommandSource NULL = new CommandSource() {};

    default void sendSystemMessage(Component message) {}

    default boolean acceptsSuccess() {
        return false;
    }

    default boolean acceptsFailure() {
        return false;
    }

    default boolean shouldInformAdmins() {
        return false;
    }
}
