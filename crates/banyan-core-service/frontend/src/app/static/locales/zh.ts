export default {
    coponents: {
        account: {
            billing: {
                invoices: {
                    title: "发票",
                    emptyStateTitle: "无支付或账单活动",
                    emptyStateDescription: "您账户上生成的发票和收据将在此处提供",
                    upgradeAccount: "升级账户",
                    manageSubscriptions: "管理订阅",
                    invoice: {
                        title: "发票",
                        summary: "摘要",
                        to: "至",
                        from: "从",
                        subscribedPlan: "已订阅计划",
                        billingPeriod: "计费周期",
                        items: "项目",
                        driveSrorage: "驱动器存储",
                        archiveStorage: "归档存储",
                        dataEggress: "数据出口",
                        payment: "支付",
                        total: "总计"
                    },
                    invoicesTable: {
                        date: "日期",
                        status: "状态",
                        totalCost: "总费用",
                        details: "详情",
                        viewInvoice: "查看发票"
                    },
                    nextBillingDate: {
                        title: "下一个计费日期",
                        onDemandStorage: "按需存储",
                        archivalStorage: "归档存储",
                        dataEggress: "数据出口",
                        totalCost: "总费用",
                        upgradeAccount: "升级账户",
                        manageSubscriptions: "管理订阅",
                    },
                }
            },
            manageKeys: {
                keyActions: {
                    removeAccess: "移除访问",
                    approveAccess: "批准访问",
                },
                keyManagementTable: {
                    device: "设备",
                    client: "客户端",
                    fingerprint: "指纹",
                    status: "状态",
                    approved: "已批准",
                    noAccess: "无访问",
                }
            },
            navigation: {
                title: "账户设置",
                profile: "个人资料",
                manageAccessKeys: "管理访问密钥",
                billingAndPayment: "账单和付款"
            },
            profile: {
                title: "个人资料",
                language: "语言",
                chooseYourLanguage: "选择您的语言"
            },
        },
        bucket: {
            bucketTable: {
                name: "名称",
                lastModified: "上次修改",
                fileSize: "文件大小",
                moveToError: "移动文件时出现问题。 请重试。",
                tryAgain: "重试",
                fileWasMoved: "文件已移动",
                uploadError: "上传时出现问题。 请重试",
                fileActions: {
                    download: "下载",
                    moveTo: "移动到",
                    makeCopy: "制作副本",
                    viewFileVersions: "查看文件版本",
                    rename: "重命名",
                    remove: "删除",
                    shareFile: "分享文件",
                    yourFileIsSecure: "您的文件已安全",
                    tryAgain: "重试",
                    downloading: "下载中",
                    fileWasDownloaded: "文件已下载",
                    copyOf: "副本",
                    wasCreated: "已创建",
                },
                folderActions: {
                    moveTo: "移动到",
                    rename: "重命名",
                    remove: "删除",
                    upload: "上传",
                },
                folderRow: {
                    tryAgain: "重试",
                    fileWasMoved: "文件已移动",
                    moveToError: "移动文件时出现问题。 请重试。",
                    uploadError: "上传时出现问题。 请重试",
                    failedToLoadFiles: "加载文件失败"
                }
            },
            emptyState: {
                description: "将文件拖放到此处以上传，或使用“上传”按钮",
                buttonText: "上传",
                tryAgain: "重试",
                uploadError: "上传时出现问题。 请重试",
            },
            header: {
                files: "文件",
                uploadButton: "上传",
                createFolderButton: "创建文件夹",
                snapshotBannerTitle: "归档快照",
                snapshotBannerSubtitle: "此驱动器没有快照",
                snapshotBannerExplanation: "什么是快照",
                snapshotBannerTooltip: "归档快照提供文件的某个时刻的快照，对版本控制很有用",
                makeSnapshot: "创建快照",
            }
        },
        home: {
            bucket: {
                hot: "按需",
                warm: "按需",
                cold: "归档",
                coldTooltip: "归档存储适用于您希望安全存储很长时间且不打算频繁访问的文件。",
                hotTooltip: "按需存储适用于您计划频繁访问并需要快速访问的文件。",
                warmTooltip: "按需存储适用于您计划频繁访问并需要快速访问的文件。",
                coldSnapshots: "归档快照",
                files: "文件",
                uploadError: "上传时出现问题。 请重试",
                tryAgain: "重试",
            },
            emptyState: {
                title: "创建您的第一个驱动器以开始上传和共享",
                newDriveButton: "新驱动器",
            },
        },
        common: {
            betaBanner: {
                title: "榕树存储仍处于测试阶段，尚未投入生产。 我们建议目前使用备份存储方法来存储在榕树上的任何内容。 如果您需要立即在生产中存储数据，请联系 tim@banyan.computer"
            },
            bucketActions: {
                upload: "上传",
                takeArchivalSnapshot: "拍摄归档快照",
                viewArchivalSnapshots: "查看归档快照",
                viewDriveVersions: "查看驱动器版本",
                rename: "重命名",
                createFolder: "创建文件夹",
                restoreCold: "恢复冷版本",
                deleteHotData: "删除热数据",
                delete: "删除",
                purgeColdKeys: "清除冷钥匙",
                unlock: "解锁",
                snapshotExplanation: "归档快照提供文件的某个时刻的快照，对版本控制很有用",
            },
            filePreview: {
                downloading: "下载中",
                fileWasDownloaded: "文件已下载",
                tryAgain: "重试",
                shareFile: "分享文件",
                fileIsNotSupported: "文件不支持预览",
                actions: {
                    moveTo: "移动到",
                    rename: "重命名",
                    remove: "删除",
                }
            },
            folderSelect: {
                createFolder: "创建文件夹",
            },
            header: {
                upgrade: "升级",
                helpControls: {
                    contactUs: "联系我们",
                },
                profileControls: {
                    settings: "设置",
                    logout: "登出",
                    upgrade: "升级",
                }
            },
            mobilePlaceholder: {
                title: "您准备好了！",
                subtitle: "要开始使用榕树，请使用桌面版",
            },
            navigation: {
                uploadError: "上传时出现问题。 请重试",
                tryAgain: "重试",
                allDrives: "所有驱动器",
                lockedTooltip: {
                    youHaveNoAccess: "您无权访问此驱动器",
                    requestAccess: "请求访问",
                    here: "这里",
                }
            },
            storageUsage: {
                storage: "存储",
                used: "已使用",
                of: "的",
                upgradePlan: "升级计划",
            },
            termsAndConditions: {
                accountTypeQuestion: "您使用榕树做什么？",
                agreeToTerms: "我同意榕树的",
                termsOf: "服务条款",
                and: "和",
                privacyPolicy: "隐私政策",
                continue: "继续",
            },
            uploadFileProgress: {
                uploading: "上传中",
                uploadFailed: "上传失败"
            },
            modal: {
                approveBucketAccess: {
                    title: "批准访问",
                    subtitle: "您确定要批准访问吗 ",
                    tryAgain: "重试",
                    cancel: "取消",
                    approveAccess: "批准访问",
                },
                bucketSnapshots: {
                    title: "查看归档快照",
                    subtitle: "访问并查看以前的版本",
                    tryAgain: "重试",
                    close: "关闭",
                },
                createBucket: {
                    createNewDrive: "创建新驱动器",
                    driveName: "驱动器名称",
                    enterNewName: "输入新名称",
                    cancel: "取消",
                    create: "创建",
                    creationError: "创建驱动器时出现问题。 请重试。",
                    tryAgain: "重试",
                },
                createFolder: {
                    title: "创建文件夹",
                    folderName: "文件夹名称",
                    enterNewName: "输入新名称",
                    cancel: "取消",
                    create: "创建",
                    creationError: "创建文件夹时出现问题。 请重试。",
                    tryAgain: "重试",
                },
                createSecretKey: {
                    title: "创建秘密密钥",
                    subtitle: "您的秘密密钥是解锁您的帐户的方法； 创建一个您可以记住并保持安全的秘密密钥 - 没有其他人可以访问它，甚至榕树，所以确保您可以记住它以解锁您的帐户。",
                    secretKey: "秘密密钥",
                    enterSecretKey: "输入秘密密钥",
                    keyRequirements: "密钥必须至少为8个字符",
                    confirmSecretKey: "确认秘密密钥",
                    passphraseNotMatch: "密码不匹配，请重试",
                    confirm: "确认",
                    creationError: "初始化密钥库失败"
                },
                deleteBucket: {
                    title: "删除驱动器",
                    subtitle: "您确定要删除",
                    filesWillBeDeletedPermanently: "文件将被永久删除",
                    delete: "删除",
                    cancel: "取消",
                    drive: "驱动器",
                    wasDeleted: "已删除",
                    deletionError: "删除时出现问题。 请重试",
                    tryAgain: "重试",
                },
                deleteFile: {
                    title: "移除文件",
                    wantToMove: "您要删除吗",
                    filesWillBeMoved: "文件将被永久删除",
                    delete: "删除",
                    cancel: "取消",
                    file: "文件",
                    wasDeleted: "已删除",
                    deletionError: "删除时出现问题。 请重试",
                    tryAgain: "重试",
                },
                enterSecretKey: {
                    title: "输入秘密密钥",
                    subtitle: "将秘密密钥输入文本字段",
                    secretKey: "秘密密钥",
                    enterSecretKey: "输入秘密密钥",
                    keyRequirements: "密钥必须至少为8个字符",
                    confirm: "确认",
                    sectretKeyError: "错误的秘密密钥",
                },
                hardStorageLimit: {
                    title: "您的存储空间不足",
                    subtitle: "升级您的帐户以上传和同步文件。",
                    cancel: "取消",
                    upgradePlan: "升级计划",
                },
                moteTo: {
                    title: "移动到",
                    subtitle: "请选择要移动文件的位置",
                    folder: "文件夹",
                    cancel: "取消",
                    moveTo: "移动到",
                    fileWasMoved: "文件已移动",
                    moveToError: "移动文件时出现问题。 请重试",
                    tryAgain: "重试",
                },
                removeBucketAccess: {
                    title: "删除访问",
                    subtitle: "删除访问意味着您将来将无法访问这些数据，但您以前下载的数据仍可供您访问，但将不再同步",
                    cancel: "取消",
                    removeAccess: "删除访问"
                },
                renameBucket: {
                    title: "重命名驱动器",
                    enterNewName: "输入新名称",
                    cancel: "取消",
                    save: "保存",
                    drive: "驱动器",
                    wasRenamed: "已重命名",
                    editError: "您的编辑出现问题。 请重试",
                    tryAgain: "重试",
                },
                renameFile: {
                    title: "重命名文件",
                    fileName: "文件名",
                    enterNewName: "输入新名称",
                    cancel: "取消",
                    save: "保存",
                    fileWasRenamed: "文件已重命名",
                    editError: "您的编辑出现问题。 请重试",
                    tryAgain: "重试",
                },
                requestBucketAccess: {
                    title: "请求访问",
                    subtitle: "您想要请求访问吗？",
                    cancel: "取消",
                    requestAccess: "请求访问",
                },
                shareFile: {
                    title: "分享",
                    cancel: "取消",
                    copyLink: "复制链接",
                    linkWasCopied: "链接已复制",
                },
                subscriptionPlan: {
                    title: "选择适合您的计划",
                    subtitle: "以实惠的价格获得您应得的存储、安全性和隐私权",
                    hotStorage: "热存储",
                    hotReplications: "热复制",
                    freeEgress: "免费出口",
                    archivalSnapshots: "归档快照",
                    litePlanDescription: "免费，但功能有限",
                    currentPlan: "当前计划",
                    upgradeTo: "升级到",
                    needCustomPlan: "需要更多定制的计划",
                    contactSales: "联系销售",
                },
                takeSnapshot: {
                    title: "拍摄归档快照",
                    subtitle: "您要拍摄归档快照吗",
                    cancel: "取消",
                    takeArchivalSnapshot: "拍摄归档快照",
                    snapshotWasTaken: "归档快照已拍摄",
                    snapshotError: "拍摄快照时出现问题。 请重试。",
                    tryAgain: "重试",
                },
                termsAndConditions: {
                    title: "服务条款和隐私政策",
                    decline: "拒绝",
                    acceptTermsAndService: "我已阅读并接受榕树的服务条款",
                },
                uploadFile: {
                    title: "上传文件",
                    subtitle: "从您的设备选择要上传的文件，或使用拖放",
                    selectDrive: "选择驱动器",
                    createNewDrive: "创建新驱动器",
                    selectFolder: "选择文件夹",
                    clickToUpload: "单击上传",
                    orDragAndDrop: "或拖放",
                    cancel: "取消",
                    upload: "上传",
                    uploadError: "上传时出现问题。 请重试",
                    tryAgain: "重试",
                }
            }
        },
    },
    pages: {
        home: {
            allDrives: "所有驱动器",
            upload: "上传",
            newDrive: "新驱动器",
        },
        registerDevice: {
            title: "批准访问",
            wantToApproveAccess: "您确定要批准访问吗",
            cancel: "取消",
            approveAccess: "批准访问",
        },
        trash: {
            trash: "垃圾桶",
            trashIsFull: "垃圾桶已满",
            emptyTrash: "点击清空垃圾桶",
            trashIsEmpty: "垃圾桶是空的",
            clickToEmptyTrash: "点击清空垃圾桶",
        }
    },
    contexts: {
        fileUpload: {
            softStorageLimit: "您接近存储计划的限制，请升级以获得更多空间",
            hardStorageLimit: "您的存储请求超出存储容量，请",
            seePricingPage: "查看我们的定价页面",
            contactSales: "联系销售"
        }
    }
};