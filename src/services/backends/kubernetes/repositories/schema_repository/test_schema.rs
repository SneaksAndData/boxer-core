use serde_json::{Value, json};

pub fn schema() -> Value {
    json!({
        "PhotoApp": {
            "commonTypes": {
                "PersonType": {
                    "type": "Record",
                    "attributes": {
                        "age": {
                            "type": "Long"
                        },
                        "name": {
                            "type": "String"
                        }
                    }
                }
            },
            "entityTypes": {
                "User": {
                    "shape": {
                        "type": "Record",
                        "attributes": {
                            "userId": {
                                "type": "String"
                            },
                            "personInformation": {
                                "type": "PersonType"
                            }
                        }
                    },
                    "memberOfTypes": [
                        "UserGroup"
                    ]
                },
                "UserGroup": {
                    "shape": {
                        "type": "Record",
                        "attributes": {}
                    }
                },
            "Photo": {
                "shape": {
                    "type": "Record",
                    "attributes": {
                        "private": {
                            "type": "Boolean",
                            "required": true
                        }
                    }
                },
                "memberOfTypes": [
                ]
            },
            },
            "actions": {
              "viewPhoto": {
                "appliesTo": {
                    "principalTypes": [
                        "User",
                        "UserGroup"
                    ],
                    "resourceTypes": [
                        "Photo"
                    ],
                }
            },
          }
        }
    })
}
