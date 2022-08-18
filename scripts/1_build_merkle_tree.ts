import { MerkleTree } from "merkletreejs";

import { sha256 } from "./hash";

type User = {
  address: string;
  amount: number;
};

// these are the test accounts used by localterra: https://github.com/terra-money/LocalTerra
const users: User[] = [
  {
    address: "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v",
    amount: 12345,
  },
  {
    address: "terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp",
    amount: 23456,
  },
  {
    address: "terra1757tkx08n0cqrw7p86ny9lnxsqeth0wgp0em95",
    amount: 42069,
  },
  {
    address: "terra199vw7724lzkwz6lf2hsx04lrxfkz09tg8dlp6r",
    amount: 88888,
  },
  {
    address: "terra18wlvftxzj6zt0xugy2lr9nxzu402690ltaf4ss",
    amount: 987654321,
  },
  {
    address: "terra1e8ryd9ezefuucd4mje33zdms9m2s90m57878v9",
    amount: 999999999
  },
  {
    address: "terra17tv2hvwpg0ukqgd2y5ct2w54fyan7z0zxrm2f9",
    amount: 4237784,
  },
  {
    address: "terra1lkccuqgj6sjwjn8gsa9xlklqv4pmrqg9dx2fxc",
    amount: 7324894,
  },
  {
    address: "terra1333veey879eeqcff8j3gfcgwt8cfrg9mq20v6f",
    amount: 42873
  },
  {
    address: "terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n",
    amount: 74328473
  },
  {
    address: "terra1u48kjlf2n0s77v0yhkmle58rdqgrqmdam0cv56",
    amount: 723443,
  },
  {
    address: "terra1sspv8dyfg2p4phxu29k28zm3t3kmx7x5ljw9ae",
    amount: 42743987,
  },
  {
    address: "terra1wdqnf2ktzcqdnm5sl20hnwrzzlfv5mswmfn52r",
    amount: 723943,
  },
  {
    address: "terra1z926ax906k0ycsuckele6x5hh66e2m4m5udwep",
    amount: 88888888
  }
];

const toLeaf = (user: User) => Buffer.from(`${user.address}:${user.amount}`, "utf8");

function generateProof(users: User[], index: number) {
  const leaves = users.map((user) => sha256(toLeaf(user)));
  const tree = new MerkleTree(leaves, sha256, { sortLeaves: false, sortPairs: true }); // IMPORTANT: the sort options
  console.log("tree:\n", tree.toString());

  const user = users[index]!;
  const leaf = sha256(toLeaf(user));
  const proof = tree.getProof(leaf).map((p) => p.data.toString("hex"));
  console.log("address:", user.address);
  console.log("amount:", user.amount);
  console.log("leaf:", leaf.toString("hex"));
  console.log("proof:", proof);

  const sum = users.map((user) => user.amount).reduce((partialSum, a) => partialSum + a, 0);
  console.log("total amount:", sum);
}

function testCase1() {
  // generate proof for the 3rd user
  generateProof(users, 2);
}

function testCase2() {
  // user 3 fabricates a different amount (69420 rather than 42069)
  let falseUsers = users;
  falseUsers[2]!.amount = 69420;

  // generate proof for the 3rd user
  generateProof(falseUsers, 2);
}

function testCase3() {
  // inserts a non-existent user
  const falseUsers = [
    ...users.slice(0, 2),
    {
      address: "terra17tv2hvwpg0ukqgd2y5ct2w54fyan7z0zxrm2f9",
      amount: 123456789,
    },
    ...users.slice(2, 6),
  ];

  // generate proof for the inserted user
  generateProof(falseUsers, 2);
}

console.log("\n********** test case 1 **********\n");
testCase1();

console.log("\n********** test case 2 **********\n");
testCase2();

console.log("\n********** test case 3 **********\n");
testCase3();
