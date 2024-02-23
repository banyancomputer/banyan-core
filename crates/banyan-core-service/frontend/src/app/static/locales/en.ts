export default {
    coponents : {
        account: {
            billing: {
                invoices: {
                    title: "Invoices",
                    emptyStateTitle: "No payment or billing activity",
                    emptyStateDescription: "Invoices and receipts made on your account will be available here",
                    upgradeAccount: "Upgrade account",
                    manageSubscriptions: "Manage subscriptions",
                    invoice: {
                        title: "Invoice",
                        summary: "Summary",
                        to: "To",
                        from: "From",
                        subscribedPlan: "Subscribed Plan",
                        billingPeriod: "Billing period",
                        items: "Items",
                        driveSrorage: "Drive storage",
                        archiveStorage: "Archive storage",
                        dataEggress: "Data egress",
                        payment: "Payment",
                        total: "Total"
                    },
                    invoicesTable: {
                        date: "Date",
                        status: "Status",
                        totalCost: "Total cost",
                        details: "Deatils",
                        viewInvoice: "View invoice"
                    },
                    nextBillingDate: {
                        title: "Next Billing Date",
                        onDemandStorage: "On Demand Storage",
                        archivalStorage: "Archival Storage",
                        dataEggress: "Data egress",
                        totalCost: "Total cost",
                        upgradeAccount: "Upgrade account",
                        manageSubscriptions: "Manage subscriptions",
                    },
                }
            },
            manageKeys: {
                keyActions: {
                    removeAccess:"Remove access",
                    approveAccess:"Approve access",
                },
                keyManagementTable: {
                    device: "Device",
                    client: "Client",
                    fingerprint: "Fingerprint",
                    status: "Status",
                    approved: "Approved",
                    noAccess: "No access",
                }
            },
            navigation: {
                title: "Account settings",
                profile: "Profile",
                manageAccessKeys: "Manage Access Keys",
                billingAndPayment: "Billing & Payment"
            },
            profile: {
                title: "Profile",
                language:"Language",
                chooseYourLanguage: "Choose your language"
            },
        },
        bucket: {
            bucketTable:{
                name: "Name",
                lastModified: "Last Modified",
                fileSize: "File size",
                moveToError: "There was an issue moving your file. Please try again.",
                tryAgain: "Try again",
                fileWasMoved: "File was moved",
                uploadError: "There was an issue with upload. Please try again",
                fileActions: {
                    download: "Download",
                    moveTo: "Move to",
                    makeCopy: "Make a copy",
                    viewFileVersions: "View file versions",
                    rename: "Rename",
                    remove: "Remove",
                    shareFile: "Share file",
                    yourFileIsSecure: "Your file is secure",
                    tryAgain: "Try again",
                    downloading: "Downloading",
                    fileWasDownloaded: "File was downloaded",
                    copyOf: "Copy of",
                    wasCreated: "was created",
                },
                folderActions: {
                    moveTo: "Move to",
                    rename: "Rename",
                    remove: "Remove",
                    upload: "Upload",
                },
                folderRow: {
                    tryAgain: "Try again",
                    fileWasMoved: "File was moved",
                    moveToError: "There was an issue moving your file. Please try again.",
                    uploadError: "There was an issue with upload. Please try again",
                    failedToLoadFiles: "Failed to load files"
                }
            },
            emptyState: {
                description: "Drag & drop files here to upload,or use the 'Upload' button",
                buttonText: "Upload",
                tryAgain: "Try again",
                uploadError: "There was an issue with upload. Please try again",
            },
            header: {
                files: "Files",
                uploadButton: "Upload",
                createFolderButton: "Create Folder",
                snapshotBannerTitle: "Archival Snapshots",
                snapshotBannerSubtitle: "This drive has no snapshots",
                snapshotBannerExplanation: "What is a snapshot",
                snapshotBannerTooltip: "Archival snapshots offer a point-in-time glimpse of the file and are useful for versioning",
                makeSnapshot: "Make a Snapshot",
            }
        },
        home: {
            bucket: {
                hot: "On-Demand",
                warm: "On-Demand",
                cold: "Archival",
                coldTooltip: "Archival storage is for files you want stored safely for a long time and don't plan to access very frequently.",
                hotTooltip: "On-demand storage is for files you plan to access frequently and require quick access to.",
                warmTooltip: "On-demand storage is for files you plan to access frequently and require quick access to.",
                coldSnapshots: "Archival Snapshots",
                files: "Files",
                uploadError: "There was an issue with upload. Please try again",
                tryAgain: "Try again",
            },
            emptyState: {
                title: "Create your first drive to start uploading and sharing",
                newDriveButton: "New drive",
            },
        },
        common: {
            betaBanner: {
                title: "Banyan Storage is still in Beta and not production-ready. We recommend using a backup storage method for anything stored on Banyan at this time. If you need to store data in Prod today, contact tim@banyan.computer"
            },
            bucketActions: {
                upload: "Upload",
                takeArchivalSnapshot: "Take a snapshot",
                viewArchivalSnapshots: "View snapshots",
                viewDriveVersions: "View drive versions",
                rename: "Rename",
                createFolder: "Create folder",
                restoreCold: "Restore cold version",
                deleteHotData: "Delete hot data",
                delete: "Delete",
                purgeColdKeys: "Purge cold keys",
                unlock: "Unlock",
                snapshotExplanation: "Archival snapshots offer a point-in-time glimpse of the file and are useful for versioning",
            },
            filePreview: {
                downloading: "Downloading",
                fileWasDownloaded: "File was downloaded",
                tryAgain: "Try again",
                shareFile: "Share file",
                fileIsNotSupported: "File is not supported for preview",
                actions: {
                    moveTo: "Move to",
                    rename: "Rename",
                    remove: "Remove",
                }
            },
            folderSelect: {
                createFolder: "Create folder",
            },
            header: {
                upgrade: "Upgrade",
                helpControls: {
                    contactUs: "Contact Us",
                },
                profileControls: {
                    settings: "Settings",
                    logout: "Log Out",
                    upgrade: "Upgrade",
                }
            },
            mobilePlaceholder: {
                title: "You’re ready to start!",
                subtitle: "To start using Banyan, please use desktop",
            },
            navigation: {
                uploadError: "There was an issue with upload. Please try again",
                tryAgain: "Try again",
                allDrives: "All Drives",
                lockedTooltip: {
                    youHaveNoAccess: "You don't have access to this drive",
                    requestAccess: "Request access",
                    here: "here",
                }
            },
            storageUsage: {
                storage: "Storage",
                used: "Used",
                of: "of",
                upgradePlan: "Upgrade Plan",
            },
            termsAndConditions: {
                accountTypeQuestion: "What are you using Banyan For?",
                agreeToTerms: "I agree to Banyan’s",
                termsOf:"terms of service",
                and: "and",
                privacyPolicy: "privacy policy",
                continue: "Continue",
            },
            uploadFileProgress: {
                uploading: "Uploading",
            },
            modal: {
                approveBucketAccess: {
                    title: "Approve access",
                    subtitle: "Are you sure you want to approve access ",
                    tryAgain: "Try again",
                    cancel: "Cancel",
                    approveAccess: "Approve access",
                },
                bucketSnapshots: {
                    title: "View archival snapshots",
                    subtitle: "Access and review previous versions",
                    tryAgain: "Try again",
                    close: "Close",
                },
                createBucket: {
                    createNewDrive: "Create new drive",
                    driveName: "Drive name",
                    enterNewName: "Enter new name",
                    cancel: "Cancel",
                    create: "Create",
                    creationError: "There was an issue creating your drive. Please try again.",
                    tryAgain: "Try again",
                },
                createFolder: {
                    title: "Create folder",
                    folderName: "Folder name",
                    enterNewName: "Enter new name",
                    cancel: "Cancel",
                    create: "Create",
                    creationError: "There was an issue creating your folder. Please try again.",
                    tryAgain: "Try again",
                },
                createSecretKey: {
                    title: "Create a secret key",
                    subtitle: "Your secret key is the way to unlock your account; create a secret key you can remember and keep it safe – no one else has access to it, not even Banyan, so make sure you can remember it to unlock your account.",
                    secretKey: "Secret Key",
                    enterSecretKey: "Enter secret key",
                    keyRequirements: "Key must be at least 8 characters",
                    confirmSecretKey: "Confirm secret key",
                    passphraseNotMatch: "The passphrases do not match, please try again",
                    confirm: "Confirm",
                    creationError: "Failed to initialize keystore"
                },
                deleteBucket: {
                    title: "Delete drive",
                    subtitle: "Are you sure you want to delete",
                    filesWillBeDeletedPermanently: "Files will be deleted permanently",
                    delete: "Delete",
                    cancel: "Cancel",
                    drive: "Drive",
                    wasDeleted: "was deleted",
                    deletionError: "There was an issue with deletion. Please try again",
                    tryAgain: "Try again",
                },
                deleteFile: {
                    title: "Remove File",
                    wantToMove: "Do you want to remove",
                    filesWillBeMoved: "File will be deleted permanently",
                    delete: "Delete",
                    cancel: "Cancel",
                    file: "File",
                    wasDeleted: "was deleted",
                    deletionError: "There was an issue with deletion. Please try again",
                    tryAgain: "Try again",
                },
                enterSecretKey: {
                    title: "Input secret key",
                    subtitle: "Enter the secret key into text field",
                    secretKey: "Secret Key",
                    enterSecretKey: "Enter secret key",
                    keyRequirements: "Key must be at least 8 characters",
                    confirm: "Confirm",
                    sectretKeyError: "Wrong secret key",
                },
                hardStorageLimit: {
                    title: "You’re out of storage",
                    subtitle: "Upgrade your account to upload and sync files.",
                    cancel: "Cancel",
                    upgradePlan: "Upgrade Plan",
                },
                moteTo: {
                    title: "Move to",
                    subtitle: "Please select where you would like to move your file",
                    folder: "Folder",
                    cancel: "Cancel",
                    moveTo: "Move to",
                    fileWasMoved: "File was moved",
                    moveToError: "There was an issue moving your file. Please try again",
                    tryAgain: "Try again",
                },
                removeBucketAccess: {
                    title: "Remove access",
                    subtitle: "Removing Access means that you will not have future access to this data, any previously downloaded data is still accessible to you but will no longer be synced",
                    cancel: "Cancel",
                    removeAccess: "Remove access"
                },
                renameBucket: {
                    title: "Rename drive",
                    enterNewName: "Enter new name",
                    cancel: "Cancel",
                    save: "Save",
                    drive: "Drive",
                    wasRenamed: "was renamed",
                    editError: "There was an issue with your edit. Please try again",
                    tryAgain: "Try again",
                },
                renameFile: {
                    title: "Rename file",
                    fileName: "File name",
                    enterNewName: "Enter new name",
                    cancel: "Cancel",
                    save: "Save",
                    fileWasRenamed: "File was renamed",
                    editError: "There was an issue with your edit. Please try again",
                    tryAgain: "Try again",
                },
                requestBucketAccess: {
                    title: "Request access",
                    subtitle: "Do you want to request access?",
                    cancel: "Cancel",
                    requestAccess: "Request access",
                },
                shareFile: {
                    title: "Share",
                    cancel: "Cancel",
                    copyLink: "Copy link",
                    linkWasCopied: "Link was copied",
                },
                subscriptionPlan: {
                    title: "Choose a plan that’s right for you",
                    subtitle: "Get the storage, security, and privacy you deserve, for an affordable price",
                    hotStorage: "Hot Storage",
                    hotReplications: "Hot Replications",
                    freeEgress: "Free egress",
                    archivalSnapshots: "Archival Snapshots",
                    litePlanDescription: "Free, but with limited features",
                    currentPlan: "Current plan",
                    upgradeTo: "Upgrade to",
                    needCustomPlan: "Need more customized plan",
                    contactSales: "Contact sales",
                },
                takeSnapshot: {
                    title: "Take a snapshot",
                    subtitle: "Do you want to take a snapshot",
                    cancel: "Cancel",
                    takeArchivalSnapshot: "Take a snapshot",
                    snapshotWasTaken: "Archival snapshot was taken",
                    snapshotError: "There was an issue with your snapshot. Please try again.",
                    tryAgain: "Try again",
                },
                termsAndConditions: {
                    title: "Terms of Service & Privacy Policy",
                    decline: "Decline",
                    acceptTermsAndService: "I have read and accept Banyan`s Terms of Service",
                },
                uploadFile: {
                    title: "Upload files",
                    subtitle: "Choose files for uploading from your device, or use drag & drop",
                    selectDrive: "Select drive",
                    createNewDrive: "Create new drive",
                    selectFolder: "Select folder",
                    clickToUpload: "Click to upload",
                    orDragAndDrop: "or drag and drop",
                    cancel: "Cancel",
                    upload: "Upload",
                    uploadError: "There was an issue with upload. Please try again",
                    tryAgain: "Try again",
                }
            }
        },
    },
    pages: {
        home: {
            allDrives: "All Drives",
            upload: "Upload",
            newDrive: "New Drive",
        },
        registerDevice: {
            "title": "Approve access",
            "wantToApproveAccess": "Are you sure you want to approve access ",
            "cancel": "Cancel",
            "approveAccess": "Approve access",
        },
        trash: {
            trash: "Trash",
            trashIsFull: "Trash is full",
            emptyTrash: "Click to empty trash",
            trashIsEmpty: "Trash is empty",
            clickToEmptyTrash: "Click to empty trash",
        }
    },
    contexts: {
        fileUpload: {
            softStorageLimit: "You're approaching the limit of your storage plan, upgrade for more space",
            hardStorageLimit: "Your storage request exceeded storage capacity, please",
            seePricingPage: "See our pricing page",
            contactSales: "Contact sales"
        }
    }
};
