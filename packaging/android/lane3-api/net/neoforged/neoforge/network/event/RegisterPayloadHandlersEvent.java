// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.network.event;

import net.neoforged.bus.api.Event;
import net.neoforged.neoforge.network.registration.PayloadRegistrar;

/** Mods name their network messages. */
public class RegisterPayloadHandlersEvent extends Event {
    public PayloadRegistrar registrar(String version) {
        return new PayloadRegistrar();
    }
}
