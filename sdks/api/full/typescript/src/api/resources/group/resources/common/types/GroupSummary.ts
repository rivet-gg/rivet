/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as Rivet from "../../../../../index";

export interface GroupSummary {
    groupId: string;
    displayName: Rivet.DisplayName;
    /** The URL of this group's avatar image. */
    avatarUrl?: string;
    external: Rivet.group.ExternalLinks;
    /**
     * **Deprecated**
     * Whether or not this group is a developer.
     */
    isDeveloper: boolean;
    bio: Rivet.Bio;
    /** Whether or not the current identity is a member of this group. */
    isCurrentIdentityMember: boolean;
    publicity: Rivet.group.Publicity;
    memberCount: number;
    ownerIdentityId: string;
}
