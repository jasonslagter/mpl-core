/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import { PublicKey } from '@metaplex-foundation/umi';
import {
  GetDataEnumKind,
  GetDataEnumKindContent,
  Serializer,
  dataEnum,
  publicKey as publicKeySerializer,
  struct,
  unit,
} from '@metaplex-foundation/umi/serializers';
import { Plugin, PluginArgs, getPluginSerializer } from '.';

export type Authority =
  | { __kind: 'Owner' }
  | { __kind: 'Permanent'; address: PublicKey }
  | { __kind: 'SameAs'; plugin: Plugin };

export type AuthorityArgs =
  | { __kind: 'Owner' }
  | { __kind: 'Permanent'; address: PublicKey }
  | { __kind: 'SameAs'; plugin: PluginArgs };

export function getAuthoritySerializer(): Serializer<AuthorityArgs, Authority> {
  return dataEnum<Authority>(
    [
      ['Owner', unit()],
      [
        'Permanent',
        struct<GetDataEnumKindContent<Authority, 'Permanent'>>([
          ['address', publicKeySerializer()],
        ]),
      ],
      [
        'SameAs',
        struct<GetDataEnumKindContent<Authority, 'SameAs'>>([
          ['plugin', getPluginSerializer()],
        ]),
      ],
    ],
    { description: 'Authority' }
  ) as Serializer<AuthorityArgs, Authority>;
}

// Data Enum Helpers.
export function authority(
  kind: 'Owner'
): GetDataEnumKind<AuthorityArgs, 'Owner'>;
export function authority(
  kind: 'Permanent',
  data: GetDataEnumKindContent<AuthorityArgs, 'Permanent'>
): GetDataEnumKind<AuthorityArgs, 'Permanent'>;
export function authority(
  kind: 'SameAs',
  data: GetDataEnumKindContent<AuthorityArgs, 'SameAs'>
): GetDataEnumKind<AuthorityArgs, 'SameAs'>;
export function authority<K extends AuthorityArgs['__kind']>(
  kind: K,
  data?: any
): Extract<AuthorityArgs, { __kind: K }> {
  return Array.isArray(data)
    ? { __kind: kind, fields: data }
    : { __kind: kind, ...(data ?? {}) };
}
export function isAuthority<K extends Authority['__kind']>(
  kind: K,
  value: Authority
): value is Authority & { __kind: K } {
  return value.__kind === kind;
}
