// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.common;

import net.neoforged.bus.api.IEventBus;

/** The game's event bus. */
public final class NeoForge {
    public static final IEventBus EVENT_BUS = dev.horizon.android.lane3.Neo.GAME;

    private NeoForge() {}
}
