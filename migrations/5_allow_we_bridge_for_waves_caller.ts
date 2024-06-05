import { invoke } from '../scripts/transaction';
import { NetworkConfig } from '../scripts/network';
import { ProofsGenerator } from '../scripts/script';
import { getAddressFromPrivateKey } from '../scripts/helpers';
import { Keypair } from '@wavesenterprise/signer';

export default async function (
  deployerSeed: string,
  network: NetworkConfig,
  proofsGenerator: ProofsGenerator
) {
  const deployerPrivateKey = await (
    await Keypair.fromExistingSeedPhrase(deployerSeed)
  ).privateKey();
  const deployerAddress = await getAddressFromPrivateKey(
    deployerPrivateKey,
    network.chainID
  );

  let wavesCallerContractAddress = '';
  let tokenWavesAdapterContractAddress = '';
  switch (network.name) {
    case 'mainnet':
      wavesCallerContractAddress = '';
      tokenWavesAdapterContractAddress = ''; // TODO
      throw 'todo';
      break;
    case 'testnet':
      wavesCallerContractAddress =
        'S5pgcet8rVx4DuubdQcwBhmzDSLLmpSWEGqF7gWd9Pt';
      tokenWavesAdapterContractAddress =
        'HSNBZeJ858vG11a7fB1x65Y5ok4cTR6RhJzcpZo7tHTU';
      break;
  }

  await invoke(
    {
      contractId: wavesCallerContractAddress,
      contractVersion: 1,
      callFunction: 'allow',
      callParams: [
        {
          type: 'string',
          key: 'caller',
          value: tokenWavesAdapterContractAddress,
        },
      ],
      callPayments: [],
    },
    deployerPrivateKey,
    network,
    proofsGenerator
  ).catch((e) => {
    throw e;
  });

  return true;
}
