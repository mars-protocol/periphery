import * as fs from "fs";
import * as path from "path";

interface GenesisState {
  app_state: {
    wasm: {
      gen_msgs: Msg[];
    };
  };
}

type MsgStoreCode = {
  store_code: {
    sender: string;
    wasm_byte_code: string; // base64-encoded bytestring
    instantiate_permission: {
      permission: "Unspecified" | "Nobody" | "OnlyAddress" | "Everybody";
      address: string;
    };
  },
};

type MsgInstantiateContract = {
  instantiate_contract: {
    sender: string;
    admin: string;
    code_id: number;
    label: string;
    msg: object;
    funds: { denom: string; amount: string; }[];
  },
};

type MsgExecuteContract = {
    execute_contract: {
    sender: string;
    contract: string;
    msg: object;
    funds: { denom: string; amount: string; }[];
  },
};

type Msg = MsgStoreCode | MsgInstantiateContract | MsgExecuteContract;

// my dev account
const devAcct = "mars1z926ax906k0ycsuckele6x5hh66e2m4m09whw6";
// the `gov` module account
const govModuleAcct = "mars10d07y265gmmuvt4z0w9aw880jnsr700j8l2urg";

const vesting = fs.readFileSync(path.join(__dirname, "../artifacts/mars_vesting.wasm"));
const vestingStr = vesting.toString("base64");

const airdrop = fs.readFileSync(path.join(__dirname, "../artifacts/mars_airdrop.wasm"));
const airdropStr = airdrop.toString("base64");


// - upload vesting code
// - instantiate vesting contract
// - create vesting allocations
// - transfer vesting ownership to gov module account
//
// - upload airdrop code
// - instantiate airdrop contract
//
// - TODO: upload cw3 code
// - TODO: instantiate cw3 multisig contracts
const msgs: Msg[] = [
  {
    store_code: {
      sender: devAcct,
      wasm_byte_code: vestingStr,
      instantiate_permission: {
        permission: "OnlyAddress",
        address: devAcct,
      },
    },
  },
  {
    store_code: {
      sender: devAcct,
      wasm_byte_code: airdropStr,
      instantiate_permission: {
        permission: "OnlyAddress",
        address: devAcct,
      },
    },
  },
  {
    instantiate_contract: {
      sender: devAcct,
      admin: govModuleAcct,
      code_id: 1,
      label: "mars/vesting",
      msg: {
        owner: devAcct,
        unlock_schedule: {
          start_time: 1646092800, // 2022-03-01
          cliff: 7776000,         // 3 months
          duration: 63072000,     // 2 years
        }
      },
      funds: [],
    },
  },
  {
    execute_contract: {
      sender: devAcct,
      contract: "mars14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9smxjtde", // vesting
      msg: {
        create_position: {
          user: devAcct,
          vest_schedule: {
            start_time: 1614556800, // 2021-03-01
            cliff: 31536000,        // 1 year
            duration: 94608000      // 3 years
          }
        }
      },
      funds: [
        {
          denom: "umars",
          amount: "10000000000000",
        },
      ],
    },
  },
  {
    execute_contract: {
      sender: devAcct,
      contract: "mars14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9smxjtde", // vesting
      msg: {
        transfer_ownership: govModuleAcct,
      },
      funds: [],
    },
  },
  {
    instantiate_contract: {
      sender: devAcct,
      admin: govModuleAcct,
      code_id: 2,
      label: "mars/airdrop",
      msg: {
        merkle_root: "a7da979c32f9ffeca6214558c560780cf06b09e52fe670f16c532b20016d7f38",
        claim_period: 0, // to test clawback, we set claim period to zero
      },
      funds: [
        {
          denom: "umars",
          amount: "1987821078",
        },
      ],
    },
  },
];

const genStatePath = path.join(process.env["HOME"]!, ".mars/config/genesis.json");
const genState: GenesisState = JSON.parse(fs.readFileSync(genStatePath, "utf8"));
genState.app_state.wasm.gen_msgs = msgs;
fs.writeFileSync(genStatePath, JSON.stringify(genState, null, 2));

console.log("gen state wrote to", genStatePath);
