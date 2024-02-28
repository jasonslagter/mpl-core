/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import { PublicKey } from '@metaplex-foundation/umi';
import {
  Serializer,
  publicKey as publicKeySerializer,
  string,
  struct,
} from '@metaplex-foundation/umi/serializers';
import {
  Key,
  KeyArgs,
  UpdateAuthority,
  UpdateAuthorityArgs,
  getKeySerializer,
  getUpdateAuthoritySerializer,
} from '.';

export type CompressionProof = {
  key: Key;
  owner: PublicKey;
  updateAuthority: UpdateAuthority;
  name: string;
  uri: string;
};

export type CompressionProofArgs = {
  key: KeyArgs;
  owner: PublicKey;
  updateAuthority: UpdateAuthorityArgs;
  name: string;
  uri: string;
};

export function getCompressionProofSerializer(): Serializer<
  CompressionProofArgs,
  CompressionProof
> {
  return struct<CompressionProof>(
    [
      ['key', getKeySerializer()],
      ['owner', publicKeySerializer()],
      ['updateAuthority', getUpdateAuthoritySerializer()],
      ['name', string()],
      ['uri', string()],
    ],
    { description: 'CompressionProof' }
  ) as Serializer<CompressionProofArgs, CompressionProof>;
}
